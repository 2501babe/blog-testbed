use std::collections::BTreeMap;
use std::mem::*;
use uuid::*;
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use serde_with::{serde_as, DisplayFromStr};
use solana_program::{
    account_info::*, entrypoint, entrypoint::ProgramResult, msg, pubkey::Pubkey,
    clock::*, program_error::*, system_instruction::*, pubkey::*, program::*,
    sysvar::rent::*, sysvar::Sysvar, log::*
};

const V5NAMESPACE: &Uuid = &Uuid::from_bytes([16, 92, 30, 120, 224, 152, 10, 207, 140, 56, 246, 228, 206, 99, 196, 138]);
const ETAG_SEED: &[u8] = "ETAG".as_bytes();
const HANDLE_WALLETS_SEED: &[u8] = "HANDLE_WALLETS".as_bytes();
const WALLET_USERDATA_SEED: &[u8] = "WALLET_USERDATA".as_bytes();
const HASHMAP_INITIAL_SIZE: u64 = 0x800;
const USERDATA_INITIAL_SIZE: u64 = 0x800;
const HANDLE_MAX_LEN: u64 = 24;

#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct HandleWallets(
    #[serde_as(as = "BTreeMap<_, DisplayFromStr>")]
    BTreeMap<Handle, Pubkey>
);
impl HandleWallets {
    fn new() -> Self {
        HandleWallets(BTreeMap::new())
    }
}
impl LoadStoreAccount for HandleWallets {}

#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct WalletUserdata(
    #[serde_as(as = "BTreeMap<DisplayFromStr, DisplayFromStr>")]
    BTreeMap<Pubkey, Pubkey>
);
impl WalletUserdata {
    fn new() -> Self {
        WalletUserdata(BTreeMap::new())
    }
}
impl LoadStoreAccount for WalletUserdata {}

// a handle is nonempty, up to 24 characters, alphanumeric and underscores
// and it cannot start with an underscore (tho ill prolly use _ for something)
// when live we need a registration fee so people dont just mass register good names
// strings are stored case sensitive but compared case insensitive
// im also considering making L and i compare equal
#[derive(Clone, Debug, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct Handle(String);
impl Handle {
    fn new(name: &str) -> Result<Self, ProgramError> {
        let chars = name.chars();

        if name.len() > 0
        && name.len() <= HANDLE_MAX_LEN as usize
        && name.is_ascii()
        && name.chars().nth(0).unwrap().is_alphanumeric()
        && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Ok(Handle(name.to_string()));
        } else {
            return Err(ProgramError::InvalidArgument);
        }
    }
}
impl PartialEq for Handle {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_ascii_lowercase() == other.0.to_ascii_lowercase()
    }
}

// FIXME character restrictions here are excessive, follow the rfc later
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Uri(String);
impl Uri {
    fn new(uri: &str) -> Result<Self, ProgramError> {
        let chars = uri.chars();

        if uri.len() > 0
        && uri.is_ascii()
        && uri.chars().nth(0).unwrap().is_alphabetic()
        && uri.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Ok(Uri(uri.to_string().to_ascii_lowercase()));
        } else {
            return Err(ProgramError::InvalidArgument);
        }
    }
}

#[serde_as]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Userdata {
    #[serde_as(as = "DisplayFromStr")]
    wallet: Pubkey,
    handle: Handle,
    display: String,
    created: UnixTimestamp,
    updated: UnixTimestamp,
    posts: Vec<Postdata>,
}
impl Userdata {
    fn new(wallet: &Pubkey, handle: &Handle, display: &str, ts: UnixTimestamp) -> Self {
        Userdata {
            wallet: *wallet,
            handle: handle.clone(),
            display: display.to_string(),
            created: ts,
            updated: ts,
            posts: [].to_vec()
        }
    }
}
impl LoadStoreAccount for Userdata {}

#[serde_as]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Postdata {
    id: PostId,
    title: String,
    uri: Uri,
    created: UnixTimestamp,
    updated: UnixTimestamp,
    #[serde_as(as = "DisplayFromStr")]
    post: Pubkey,
}
impl Postdata {
    fn new(owner: &Pubkey, title: &str, uri: &Uri, ts: UnixTimestamp, post: &Pubkey) -> Self {
        let id = PostId::new(owner, title, ts);
        Postdata { id: id, title: title.to_string(), uri: uri.clone(), created: ts, updated: ts, post: *post }
    }
}
impl LoadStoreAccount for Postdata {}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct PostId(Uuid);
impl PostId {
    // this is probably "good enough"
    fn new(wallet: &Pubkey, title: &str, ts: UnixTimestamp) -> Self {
        let wab = &wallet.to_bytes();
        let tib = title.as_bytes();
        let tsb = &ts.to_be_bytes();
        let mut vec = vec![];
        vec.extend_from_slice(wab);
        vec.extend_from_slice(tib);
        vec.extend_from_slice(tsb);
        PostId(Uuid::new_v5(V5NAMESPACE, &vec))
    }
}

trait LoadStoreAccount: Sized + Serialize + DeserializeOwned {
    fn load(account_info: &AccountInfo) -> Result<Self, ProgramError> {
        let buf = &account_info.try_borrow_data()?;
        let nul = buf.iter().position(|b| *b == 0).unwrap_or(buf.len());

        match serde_json::from_slice(&buf[0..nul]) {
            Ok(s) => Ok(s),
            Err(_) => Err(ProgramError::InvalidInstructionData),
        }
    }

    fn store(&self, account_info: &AccountInfo) -> ProgramResult {
        let mut buf = account_info.try_borrow_mut_data()?;
        let txt = match serde_json::to_string(self) {
            Ok(s) => s,
            Err(_) => return Err(ProgramError::InvalidInstructionData),
        };
        let len = txt.as_bytes().len();

        if buf.len() <= len {
            return Err(ProgramError::AccountDataTooSmall);
        }

        buf[0..len].copy_from_slice(txt.as_bytes());
        buf[len] = 0;

        Ok(())
    }
}

#[repr(C)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum ProgramInstruction {
    // set up data structs and shit
    // 0: paypig
    // 1: system program
    // 2: rent sysvar
    // 3: etag
    // 4: handles -> wallets
    // 5: wallets -> userdatas
    Initialize,

    // create a new user
    // 0: wallet
    // 1: system program
    // 2: rent sysvar
    // 3: clock sysvar
    // 4: etag
    // 5: handles -> wallets
    // 6: wallets -> userdatas
    // 7: fresh userdata address
    CreateUser {
        handle: Handle,
        display: String,
    },

    // create new postdata
    // 0: wallet
    // 1: system program
    // 2: rent sysvar
    // 3: clock sysvar
    // 4: etag
    // 5: wallet userdata address
    // 6: filled  post address
    CreatePostdata {
        title: String,
        uri: Uri,
    },

    // create new post
    // 0: wallet
    // 1: system program
    // 2: rent sysvar
    // 3: clock sysvar
    // 4: etag
    // 5: fresh post address
    CreatePost {
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

// set up a new program derived account from a given seed
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

    // confirm seeded address is correct and account is uninitialized
    if addr != *dest.key { return Err(ProgramError::InvalidAccountData); }
    if !dest.data_is_empty() { return Err(ProgramError::AccountAlreadyInitialized); }

    let seed = &[&[seedword, &borrow_pls][..]];
    let rent = rentier.minimum_balance(size as usize);
    let ix = create_account(payer.key, &addr, rent, size, program_id);

    msg!("allocating {}", addr);
    invoke_signed(&ix, accounts, seed)
}

// we store a u64 in its own account and increment it every time we modify storage
// so downstream can just check the etag before pulling datas
fn increment_etag(acct: &AccountInfo) -> ProgramResult {
    let mut etag = acct.try_borrow_mut_data()?;
    let mut dst = [0; 8];
    dst.clone_from_slice(&etag[0..8]);
    etag[0..8].copy_from_slice(&(u64::from_be_bytes(dst) + 1).to_be_bytes());

    Ok(())
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
    let etag = next_account_info(account_info_iter)?;
    let user_wallets_acct = next_account_info(account_info_iter)?;
    let wallet_users_acct = next_account_info(account_info_iter)?;

    // alloc and init program derived accounts for metadata
    // these will autofail if the accounts already exist or if the provided addresses differ
    alloc_account(accounts, program_id, payer, rentier, etag, ETAG_SEED, 8)?;
    alloc_account(accounts, program_id, payer, rentier, user_wallets_acct, HANDLE_WALLETS_SEED, HASHMAP_INITIAL_SIZE)?;
    alloc_account(accounts, program_id, payer, rentier, wallet_users_acct, WALLET_USERDATA_SEED, HASHMAP_INITIAL_SIZE)?;

    // etag is a counter and accounts are calloced so no init needed

    // handles to wallet addresses
    let mut user_wallets = user_wallets_acct.try_borrow_mut_data()?;
    user_wallets[0..2].copy_from_slice("{}".as_bytes());

    // wallet addresses to userdata addresses
    let mut wallet_users = wallet_users_acct.try_borrow_mut_data()?;
    wallet_users[0..2].copy_from_slice("{}".as_bytes());

    Ok(())
}

// create userdata for the callers wallet
fn create_user(
    accounts: &[AccountInfo],
    program_id: &Pubkey,
    handle: &Handle,
    display: &str,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let payer = next_account_info(account_info_iter)?;
    let sys = next_account_info(account_info_iter)?;
    let rentier = &Rent::from_account_info(next_account_info(account_info_iter)?)?;
    let clock = &Clock::from_account_info(next_account_info(account_info_iter)?)?;
    let etag_acct = next_account_info(account_info_iter)?;
    let user_wallets_acct = next_account_info(account_info_iter)?;
    let wallet_users_acct = next_account_info(account_info_iter)?;
    let userdata_acct = next_account_info(account_info_iter)?;

    let mut user_wallets = HandleWallets::load(user_wallets_acct)?;
    let mut wallet_users = WalletUserdata::load(wallet_users_acct)?;

    // XXX is there a way to return non shit errors?
    // check if handle is already taken
    if user_wallets.0.contains_key(handle) {
        msg!("handle already taken");
        return Err(ProgramError::InvalidArgument);
    }

    // check if user already has an account set up
    if wallet_users.0.contains_key(payer.key) {
        msg!("user account exist");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    // XXX idk how big to make this but it ought to be reallocable
    // allocate provided address for userdata
    let rent = rentier.minimum_balance(USERDATA_INITIAL_SIZE as usize);
    let ix = create_account(payer.key, userdata_acct.key, rent, USERDATA_INITIAL_SIZE, program_id);
    invoke(&ix, accounts)?;

    // build userdata and store in account
    let ts = clock.unix_timestamp;
    let userdata_struct = Userdata::new(payer.key, handle, display, ts);
    userdata_struct.store(&userdata_acct)?;

    // XXX make insert a method that returns a program result mb
    // add references to our metadata maps
    user_wallets.0.insert(handle.clone(), *payer.key);
    wallet_users.0.insert(*payer.key, *userdata_acct.key);
    user_wallets.store(&user_wallets_acct)?;
    wallet_users.store(&wallet_users_acct)?;

    // update etag and return
    increment_etag(&etag_acct);

    Ok(())
}

// create postdata for a post, i split this in two for dumb compute budget
// FIXME this is not a working solution this is just a proof of concept LOL
// as i have discovered today...
// * the 200k solana compute budget is TINY so complex operations must be broken up
// * solana transactions also have a 1232 byte size limit
// this means we need a way for the client to incrementally upload data
// it also means i cannot scalably use hashmaps stored in accounts
// and i need to get clever with derived addresses instead
fn create_postdata(
    accounts: &[AccountInfo],
    program_id: &Pubkey,
    title: &str,
    uri: &Uri,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let payer = next_account_info(account_info_iter)?;
    let sys = next_account_info(account_info_iter)?;
    let rentier = &Rent::from_account_info(next_account_info(account_info_iter)?)?;
    let clock = &Clock::from_account_info(next_account_info(account_info_iter)?)?;
    let etag_acct = next_account_info(account_info_iter)?;
    let userdata_acct = next_account_info(account_info_iter)?;
    let post_acct = next_account_info(account_info_iter)?;

    // FIXME checking that userdata is in wallet_users is too expensive
    // this should be a derived address
    let mut userdata = Userdata::load(userdata_acct)?;

    // now store post data. presently it goes in userdata so nbd
    let ts = clock.unix_timestamp;
    let postdata = Postdata::new(payer.key, title, uri, ts, post_acct.key);
    userdata.posts.push(postdata);
    userdata.store(userdata_acct);

    increment_etag(&etag_acct);

    Ok(())
}

fn create_post(
    accounts: &[AccountInfo],
    program_id: &Pubkey,
    text: &str,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let payer = next_account_info(account_info_iter)?;
    let sys = next_account_info(account_info_iter)?;
    let rentier = &Rent::from_account_info(next_account_info(account_info_iter)?)?;
    let clock = &Clock::from_account_info(next_account_info(account_info_iter)?)?;
    let etag_acct = next_account_info(account_info_iter)?;
    let post_acct = next_account_info(account_info_iter)?;

    // TODO support and validate markdown
    let text_bytes = text.as_bytes();
    let post_len = text_bytes.len();
    let rent = rentier.minimum_balance(post_len);
    let ix = create_account(payer.key, post_acct.key, rent, post_len as u64, program_id);
    invoke(&ix, accounts)?;

    let mut post_buf = post_acct.try_borrow_mut_data()?;
    post_buf[0..post_len].copy_from_slice(text_bytes);

    Ok(())
}

entrypoint!(dispatch);
fn dispatch(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let insn = ProgramInstruction::unpack(instruction_data)?;

    match insn {
        ProgramInstruction::Initialize => initialize_program(accounts, program_id),
        ProgramInstruction::CreateUser{handle, display} => create_user(accounts, program_id, &handle, &display),
        ProgramInstruction::CreatePostdata{title, uri} => create_postdata(accounts, program_id, &title, &uri),
        ProgramInstruction::CreatePost{text} => create_post(accounts, program_id, &text),
        _ => panic!("fix me"),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn handles() {
        assert!(Handle::new("").is_err());
        assert!(Handle::new("_").is_err());
        assert!(Handle::new("___").is_err());
        assert!(Handle::new("1").is_ok());
        assert!(Handle::new("123").is_ok());
        assert!(Handle::new("a").is_ok());
        assert!(Handle::new("A").is_ok());
        assert!(Handle::new("sajhdASDJSA123____").is_ok());
    }

    #[test]
    fn serialize() {
        let key1 = Pubkey::new_unique();
        let key2 = Pubkey::new_unique();
        let mut wudat = WalletUserdata::new();
        wudat.0.insert(key1, key2);
        let wudat_txt = serde_json::to_string(&wudat);
        assert!(wudat_txt.is_ok());
        let wudat_re: Result<WalletUserdata, serde_json::Error> = serde_json::from_str(&wudat_txt.unwrap());
        assert!(wudat_re.is_ok());
        assert_eq!(wudat, wudat_re.unwrap());
    }

    #[test]
    fn deserialize() {
        let init_insn = r#"{ "Initialize": null}"#;
        let init_deser: Result<ProgramInstruction, serde_json::Error> = serde_json::from_str(init_insn);
        assert!(init_deser.is_ok());

        let mkuser_insn = r#"{ "CreateUser": { "handle": "hana", "display": "hanaaa" }}"#;
        let mkuser_deser: Result<ProgramInstruction, serde_json::Error> = serde_json::from_str(mkuser_insn);
        assert!(mkuser_deser.is_ok());
    }
}
