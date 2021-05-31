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
const USERNAME_MAX_LEN: u64 = 32;

// XXX i tink the impl for these can be a trait? no ^c^v
#[derive(Clone, Debug, Serialize, Deserialize)]
struct UsernameWallets(BTreeMap<Username, Pubkey>);
impl UsernameWallets {
    fn new() -> Self {
        UsernameWallets(BTreeMap::new())
    }

    fn from_account_info(account_info: &AccountInfo) -> Result<Self, ProgramError> {
        let buf = &account_info.data.borrow();
        let nul = buf.iter().position(|b| *b == 0).unwrap_or(buf.len());

        match serde_json::from_slice(&buf[0..nul]) {
            Ok(s) => Ok(s),
            Err(_) => Err(ProgramError::InvalidInstructionData),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct WalletUserData(BTreeMap<Pubkey, Pubkey>);
impl WalletUserData {
    fn new() -> Self {
        WalletUserData(BTreeMap::new())
    }

    fn from_account_info(account_info: &AccountInfo) -> Result<Self, ProgramError> {
        let buf = &account_info.data.borrow();
        let nul = buf.iter().position(|b| *b == 0).unwrap_or(buf.len());

        match serde_json::from_slice(&buf[0..nul]) {
            Ok(s) => Ok(s),
            Err(_) => Err(ProgramError::InvalidInstructionData),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct Username(String);
impl Username {
    fn new(name: &str) -> Result<Self, ProgramError> {
        let chars = name.chars();

        if name.len() > 0
        && name.len() <= USERNAME_MAX_LEN as usize
        && name.is_ascii()
        && name.chars().nth(0).unwrap().is_alphabetic()
        && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Ok(Username(name.to_string()));
        } else {
            return Err(ProgramError::InvalidArgument);
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct UserData {
    wallet: Pubkey,
    username: Username,
    created: UnixTimestamp,
    updated: UnixTimestamp,
    posts: Vec<PostData>,
}
impl UserData {
    fn new(wallet: &Pubkey, username: &Username, ts: UnixTimestamp) -> Self {
        UserData { wallet: *wallet, username: username.clone(), created: ts, updated: ts, posts: [].to_vec() }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct PostData {
    id: Uuid,
    title: String,
    uri: String,
    created: UnixTimestamp,
    updated: UnixTimestamp,
    post: Pubkey,
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
    let ix = create_account(payer.key, &addr, rent, size, program_id);

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
    let user_wallets_acct = next_account_info(account_info_iter)?;
    let wallet_users_acct = next_account_info(account_info_iter)?;

    // alloc and init two program derived accounts for metadata mappings
    // these will autofail if the accounts already exist or if the provided addresses differ
    alloc_account(accounts, program_id, payer, rentier, user_wallets_acct, USERNAME_WALLETS_SEED, HASHMAP_INITIAL_SIZE)?;
    alloc_account(accounts, program_id, payer, rentier, wallet_users_acct, WALLET_USERDATA_SEED, HASHMAP_INITIAL_SIZE)?;

    // usernames to wallet addresses
    let mut user_wallets = user_wallets_acct.try_borrow_mut_data()?;
    user_wallets[0..2].copy_from_slice("{}".as_bytes());

    // wallet addresses to userdata addresses
    let mut wallet_users = wallet_users_acct.try_borrow_mut_data()?;
    wallet_users[0..2].copy_from_slice("{}".as_bytes());

    Ok(())
}

fn create_user(
    accounts: &[AccountInfo],
    program_id: &Pubkey,
    username: &Username,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    msg!("HANA next accounts");
    let payer = next_account_info(account_info_iter)?;
    let sys = next_account_info(account_info_iter)?;
    let clock = &Clock::from_account_info(next_account_info(account_info_iter)?)?;
    let user_wallets_acct = next_account_info(account_info_iter)?;
    let wallet_users_acct = next_account_info(account_info_iter)?;
    let userdata_acct = next_account_info(account_info_iter)?;

    msg!("HANA from account info1");
    let user_wallets = UsernameWallets::from_account_info(user_wallets_acct)?;
    msg!("HANA from account info2");
    let wallet_users = WalletUserData::from_account_info(wallet_users_acct)?;

    // XXX is there a way to return non shit errors?
    if user_wallets.0.contains_key(username) {
        msg!("HANA username taken: {:?}", username);
        return Err(ProgramError::InvalidArgument);
    }

    if wallet_users.0.contains_key(payer.key) {
        msg!("HANA user already exists: {:?}", payer.key);
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    msg!("HANA create account");
    // XXX idk how big to make this but it ought to be reallocable
    let ix = create_account(payer.key, userdata_acct.key, 0, 0x1000, program_id);
    msg!("HANA invoke");
    invoke(&ix, accounts)?;

    msg!("HANA borro");
    let mut userdata = userdata_acct.try_borrow_mut_data()?;
    let userdata_struct = UserData::new(payer.key, username, 0);
    msg!("HANA userdata: {:?}", userdata_struct);

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
        ProgramInstruction::CreateUser{username} => create_user(accounts, program_id, &username),
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
