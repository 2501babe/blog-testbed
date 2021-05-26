use std::collections::HashMap;
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
struct UsernameWallets {
    username_wallets: HashMap<String, Pubkey>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct WalletUserData {
    wallet_userdata: HashMap<Pubkey, Pubkey>,
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

fn create_hashmap(
    accounts: &[AccountInfo],
    payer_key: &Pubkey,
    this_key: &Pubkey,
    sys_key: &Pubkey,
    dest: &AccountInfo,
    seedword: &[u8],
) -> ProgramResult {
    let (addr, ctr) = Pubkey::find_program_address(&[seedword], this_key);
    let borrow_pls = [ctr];

    if addr != *dest.key { return Err(ProgramError::InvalidAccountData); }
    if !dest.data_is_empty() { return Err(ProgramError::AccountAlreadyInitialized); }

    let seed = &[&[seedword, &borrow_pls][..]];
    let ix = create_account(payer_key, &addr, 0, 0x2000, this_key);

    invoke_signed(&ix, accounts, seed)

    // TODO next i need to figure out how to write data lolz
    // make sure i can load and store the hashmaps
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

    create_hashmap(accounts, payer.key, program_id, sys.key, user_wallets, USERNAME_WALLETS_SEED)?;
    create_hashmap(accounts, payer.key, program_id, sys.key, wallet_users, WALLET_USERDATA_SEED)?;

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
