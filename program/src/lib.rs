use std::collections::BTreeMap;
use std::mem::*;
use uuid::*;
use serde::{Serialize, Deserialize};
use solana_program::{
    account_info::*, entrypoint, entrypoint::ProgramResult, msg, pubkey::Pubkey,
    clock::*, program_error::*, system_instruction::*, pubkey::*, program::*,
};

const USERNAME_WALLETS_SEED: &[u8] = "USERNAME_WALLETS".as_bytes();
const WALLET_USERDATA_SEED: &[u8] = "WALLET_USERDATA".as_bytes();

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct UsernameWallets(BTreeMap<String, Pubkey>);
impl UsernameWallets {
    fn new() -> UsernameWallets {
        UsernameWallets(BTreeMap::new())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct WalletUserData {
    wallet_userdata: BTreeMap<Pubkey, Pubkey>,
}

#[repr(C)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum ProgramInstruction {
    // set up data structs and shit
    // 0: paypig
    // 1: system program
    // 2: rent sysvar
    // 3: usernames -> wallets
    // 4: wallets -> userdatas
    Initialize,

    // create a new user
    // 0: wallet
    // 1: system program
    // 2: clock sysvar
    // 3: usernames -> wallets
    // 4: wallets -> userdatas
    // 5: fresh userdata address
    CreateUser {
        username: String,
    },

    // create a new post
    // 0: wallet
    // 1: system program
    // 2: clock sysvar
    // 3: usernames -> wallets
    // 4: wallets -> userdatas
    // 5: fresh postdata address
    // 6: fresh post address
    CreatePost {
        title: String,
        text: String,
    },
}
impl ProgramInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        if input.len() == 0 {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(match input[0] {
            0 => Self::Initialize,
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }
}

fn alloc_account(
    accounts: &[AccountInfo],
    payer_key: &Pubkey,
    this_key: &Pubkey,
    sys_key: &Pubkey,
    dest: &AccountInfo,
    seedword: &[u8],
    size: u64,
) -> ProgramResult {
    let (addr, ctr) = Pubkey::find_program_address(&[seedword], this_key);
    let borrow_pls = [ctr];

    if addr != *dest.key { return Err(ProgramError::InvalidAccountData); }
    if !dest.data_is_empty() { return Err(ProgramError::AccountAlreadyInitialized); }

    let seed = &[&[seedword, &borrow_pls][..]];
    let ix = create_account(payer_key, &addr, 0, size, this_key);

    invoke_signed(&ix, accounts, seed)
}

// set up base data structures
fn initialize_program(
    accounts: &[AccountInfo],
    program_id: &Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let payer = next_account_info(account_info_iter)?;
    let sys = next_account_info(account_info_iter)?;
    let rent = next_account_info(account_info_iter)?;
    let user_wallets = next_account_info(account_info_iter)?;
    let wallet_users = next_account_info(account_info_iter)?;

    alloc_account(accounts, payer.key, program_id, sys.key, user_wallets, USERNAME_WALLETS_SEED, 0x2000)?;
    alloc_account(accounts, payer.key, program_id, sys.key, wallet_users, WALLET_USERDATA_SEED, 0x2000)?;

    let mut uw_dat = user_wallets.try_borrow_mut_data()?;
    msg!("HANA new hmap...");
    let mut uw_map = UsernameWallets::new();
    uw_map.0.insert("hana".to_string(), *payer.key);
    msg!("HANA bmap: {:?}", uw_map);
    let map_txt = serde_json::to_string(&uw_map).unwrap();
    msg!("HANA bmap text: {:?}", map_txt);
    let map_again: Result<UsernameWallets, serde_json::Error> = serde_json::from_str(&map_txt);
    msg!("HANA bmap once again: {:?}", map_again);

    Ok(())
}

entrypoint!(dispatch);
fn dispatch(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let insn = ProgramInstruction::unpack(instruction_data)?;
    msg!("HANA insn: {:?}", insn);

    match insn {
        ProgramInstruction::Initialize => initialize_program(accounts, program_id),
        _ => panic!("fix me"),
    };

    Ok(())
}
