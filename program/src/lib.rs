use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg, pubkey::Pubkey,
};
use std::collections::HashMap;
use uuid::*;
use solana_program::clock::*;
use solana_program::program_error::*;

solana_program::declare_id!("HanaYv11111111111111111111111111111111111111");

const V5NAMESPACE: &Uuid = &Uuid::from_bytes([16, 92, 30, 120, 224, 152, 10, 207, 140, 56, 246, 228, 206, 99, 196, 138]);
const CREATE_USER: u8 = 1;
const UPDATE_USER: u8 = 2;
const CREATE_POST: u8 = 3;
const UPDATE_POST: u8 = 4;

/*
  program api
  - create account
  - update account info
  - create post and metadata
  - update post and metadata
  for now posts are utf8. validate later. allow and validate markdown later
  eventually id like some kind of drafts system
  dont worry about rellocation for now, probably abstract over it later

  client queries
  - get account info by pubkey
  - get pubkey by username
  - get post and metadata by id
  - get list of all post metadata by pubkey
  these go directly from js to jsonrpc node
  getAccountInfo, getMultipleAccounts, getProgramAccounts

  data structures
  - hashmap: username -> account pubkey
  - hashmap: account pubkey -> account info
  - hashmap: account pubkey -> all post metadata
  - hashmap: post uuid -> post address
  i thought of having a post id be an account address but it would need to be a pointer
  which is kind of useless if i need to traverse it doing some tricky shit anyway?
  account -> all posts data (includes id and address)
  and then from there i can fetch text for posts i care about
  or if i have id then id -> address wait hold on fuck
  aaargh most of these need to be 
  OK STOP THINK. post is utf8. metadata is a json blob
  account info is a json blob. all these thisngs acn go in their own indiv accounts
  then we need mappings
  * username -> wallet pubkey
  * wallet pubkey -> userdata address
  * wallet pubkey -> head of post metadata linked list
  * post uri -> post id
  * post id -> post text address
  skip username and uri for now. we also need
  * permanent address containing all hashmap addresses
  on startup we fetch that address, then fetch the hashmaps
  these are just all json ig idk. define in rust and use serde or whatever

  i dunno if post id should be an account address or not
  it would need to be a pointer if so. i should prolly use a uuid
  i dont know if its possible to get entropy onchain. could just make client submit one
  ooh we could generate a v5 uuid (shasum) from the transaction signature tho

  data will need to be realloced somehow as they grow
  - user passes in one permanent address containing addresses of program accounts
    program reinvokes itself with whichever accounts it needs
  - client continuously fetches list of accounts that it may need
    presumably as a structured data thing so it can selectively submit only those it needs
  - everything stored in one big buffer, client holds a pointer to it
    very simple to realloc but still need to traverse pointer

  ok no i got it, heres how adding to hashmaps should work
  - load map, decode
  - add item to it
  - encode
  - charge user for the space diff, store in program controlled account
  - if it fits in the account just store it
  - if it doesnt fit then create a new account, store it there
    store the new account address in the previous account
    deallocate the previous previous account
  thus we avoid (most?) race conditions
  if a client has a stale address, uhh oh wait annoying we cant just load it
  because of the stupid prefetcher. we would need to recurse
  it would be simple tho, no signature even needed
  program issues the same instruction but splices in a different account
  itd also be a good way to do upgrades if we change the data model, simple redirect
  dont worry about all this now tho

  OK COOL WE GOOD WHAT are my next steps
  - define my structs and hashmaps including json transforms
  - write a program init function to set up necessary accounts
  - write create/update account/post program functions 
  - write client get functions

  aaaaa ok this is so confusing how does the interface even work
  i have four things i wanna do. create user, update user, create post, update post
  create user takes a desired username plus whatever else stuff
  it seems message doesnt implicitly pass in th e calling account
  i need the calling account to be mutable and signed for
  its pubkey goes in the userdata and i need to credit their account
*/

type StringMap = HashMap<String, String>;

struct User {
    wallet: Pubkey,
    username: String,
    created: UnixTimestamp,
    updated: UnixTimestamp,
}

struct PostData {
    id: Uuid,
    title: String,
    uri: String,
    created: UnixTimestamp,
    updated: UnixTimestamp,
}

fn create_user(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("create user! {:?}", instruction_data);
    Ok(())
}

fn create_post(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("create post! {:?}", instruction_data);
    Ok(())
}

entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!(
        "process_instruction: {}: {} accounts, data={:?}",
        program_id,
        accounts.len(),
        instruction_data
    );

    Ok(())

/*
    match instruction_data[0] {
        CREATE_USER => create_user(program_id, accounts, &instruction_data[1..]),
        CREATE_POST => create_post(program_id, accounts, &instruction_data[1..]),
        _ => Err(ProgramError::InvalidInstructionData),
    }
*/
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn hana_zone() {
        println!("HANA DEV TEST ZONE\n----\n");

        let json_ex = r#"{ "name": "hana", "job": "troublemaker" }"#;
        let mut hmap = serde_json::from_str::<StringMap>(json_ex).unwrap();

        println!("hmap initial: {:?}", hmap);

        hmap.insert("location".to_string(), "the internet".to_string());

        println!("hmap post: {:?}", hmap);

        let post_id = Uuid::new_v5(V5NAMESPACE, "hello".as_bytes());
        println!("uuid: {:?}", post_id);

    }

    /*
    use {
        super::*,
        assert_matches::*,
        solana_program::instruction::{AccountMeta, Instruction},
        solana_program_test::*,
        solana_sdk::{signature::Signer, transaction::Transaction},
    };

    #[tokio::test]
    async fn test_transaction() {
        let program_id = Pubkey::new_unique();

        let (mut banks_client, payer, recent_blockhash) = ProgramTest::new(
            "bpf_program_template",
            program_id,
            processor!(process_instruction),
        )
        .start()
        .await;

        let mut transaction = Transaction::new_with_payer(
            &[Instruction {
                program_id,
                accounts: vec![AccountMeta::new(payer.pubkey(), false)],
                data: vec![1, 2, 3],
            }],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer], recent_blockhash);

        assert_matches!(banks_client.process_transaction(transaction).await, Ok(()));
    }
    */
    
}
