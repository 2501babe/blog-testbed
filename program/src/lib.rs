use std::collections::BTreeMap;
use std::mem::*;
use uuid::*;
use serde::{Serialize, Deserialize};
use solana_program::{
    account_info::*, entrypoint, entrypoint::ProgramResult, msg, pubkey::Pubkey,
    clock::*, program_error::*, system_instruction::*, pubkey::*, program::*,
    sysvar::rent::*, sysvar::Sysvar,
};

const USERNAME_WALLETS_SEED: &[u8] = "USERNAME_WALLETS".as_bytes();
const WALLET_USERDATA_SEED: &[u8] = "WALLET_USERDATA".as_bytes();
const HASHMAP_INITIAL_SIZE: u64 = 0x2000;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct UsernameWallets(BTreeMap<String, Pubkey>);
impl UsernameWallets {
    fn new() -> UsernameWallets {
        UsernameWallets(BTreeMap::new())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct WalletUserData(BTreeMap<Pubkey, Pubkey>);
impl WalletUserData {
    fn new() -> WalletUserData {
        WalletUserData(BTreeMap::new())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Username(String);
impl Username {
    fn new(name: &str) -> Result<Username, ProgramError> {
        let chars = name.chars();

        if name.len() > 0
        && name.len() < 32
        && name.is_ascii()
        && name.chars().nth(0).unwrap().is_alphabetic()
        && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Ok(Username(name.to_string()));
        } else {
            return Err(ProgramError::InvalidArgument);
        }
    }
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
        username: Username,
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
        match serde_json::from_slice(input) {
            Ok(s) => Ok(s),
            Err(_) => Err(ProgramError::InvalidInstructionData),
        }
    }
}

fn alloc_account(
    accounts: &[AccountInfo],
    program_id: &Pubkey,
    payer: &AccountInfo,
    rentier: &Rent,
    dest: &AccountInfo,
    seedword: &[u8],
    size: u64,
) -> ProgramResult {
    let (addr, ctr) = Pubkey::find_program_address(&[seedword], program_id);
    let borrow_pls = [ctr];

    if addr != *dest.key { return Err(ProgramError::InvalidAccountData); }
    if !dest.data_is_empty() { return Err(ProgramError::AccountAlreadyInitialized); }

    let seed = &[&[seedword, &borrow_pls][..]];
    let rent = rentier.minimum_balance(size as usize);
    let ix = create_account(payer.key, &addr, 0, size, program_id);

    msg!("allocating {}", addr);
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
    let rentier = &Rent::from_account_info(next_account_info(account_info_iter)?)?;
    let user_wallets = next_account_info(account_info_iter)?;
    let wallet_users = next_account_info(account_info_iter)?;

    alloc_account(accounts, program_id, payer, rentier, user_wallets, USERNAME_WALLETS_SEED, HASHMAP_INITIAL_SIZE)?;
    alloc_account(accounts, program_id, payer, rentier, wallet_users, WALLET_USERDATA_SEED, HASHMAP_INITIAL_SIZE)?;

    let mut uw_data = user_wallets.try_borrow_mut_data()?;
    uw_data[0..2].copy_from_slice("{}".as_bytes());

    let mut wu_data = wallet_users.try_borrow_mut_data()?;
    wu_data[0..2].copy_from_slice("{}".as_bytes());

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
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn usernames() {
        assert!(Username::new("").is_err());
        assert!(Username::new("_").is_err());
        assert!(Username::new("___").is_err());
        assert!(Username::new("1").is_err());
        assert!(Username::new("123").is_err());
        assert!(Username::new("a").is_ok());
        assert!(Username::new("A").is_ok());
        assert!(Username::new("sajhdASDJSA123____").is_ok());
    }

    #[test]
    fn deserialize() {
        let init_insn = r#"{ "Initialize": null}"#;
        let init_deser: Result<ProgramInstruction, serde_json::Error> = serde_json::from_str(init_insn);
        assert!(init_deser.is_ok());

        let mkuser_insn = r#"{ "CreateUser": { "username": "hana" }}"#;
        let mkuser_deser: Result<ProgramInstruction, serde_json::Error> = serde_json::from_str(mkuser_insn);
        assert!(mkuser_deser.is_ok());
    }
}
