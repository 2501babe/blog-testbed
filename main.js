"use strict";

const w3 = require("@solana/web3.js");
const fs = require("fs").promises;
const path = require("path");

// constants
const LAMPS = 1000000000;
const KEYFILE = "testwallet.bin";
const NETWORK = "http://fortuna:8899";
const PROGRAM_ID = new w3.PublicKey("AbBrxmZKUJdn5ezmUUQSjefwojspSNSFwUDCHajg8H79");

// for debug these should be processed (faster) and true (actually test the server)
// but in prod we prolly want finalized and false
const COMMITMENT = "processed";
const SKIP_PREFLIGHT = true;

const username_regex = /^[a-zA-Z][a-zA-Z0-9_]{0,31}$/;

// debauched solweb3 devs use an async sha256 so these cant be toplevel constants
var usernameWalletAddr;
var walletUserDataAddr;

// basic program shit
const main = {

    connect: (network) => {
        let conn = new w3.Connection(network);
        return conn;
    },

    wallet: async (conn) => {
        let privkey = await fs.readFile(KEYFILE).catch(() => null);
        let wallet = new w3.Account(privkey);

        if(!privkey) {
            await conn.requestAirdrop(wallet.publicKey, 10 * LAMPS);
            await fs.writeFile(KEYFILE, wallet.secretKey);
        }

        console.log("wallet address:", wallet.publicKey.toString());
        return wallet;
    },

};

// jsonrpc api
const get = {

    // XXX there is probably a less stupid way to do this
    struct: async (conn, addr) => {
        let acct = await conn.getAccountInfo(addr, COMMITMENT);
        let str = acct ? acct.data.toString().split("\0").shift() : "";
        return str.length > 0 ? JSON.parse(str) : {};
    },

}

// rust api
const post = {

    // {"Initialize": null}
    initialize: async (conn, wallet) => {
        let data = Buffer.from('{"Initialize": null}', "utf8");

        let keys = [
            {pubkey: wallet.publicKey, isSigner: true, isWritable: true},
            {pubkey: w3.SystemProgram.programId, isSigner: false, isWritable: false},
            {pubkey: w3.SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false},
            {pubkey: usernameWalletAddr, isSigner: false, isWritable: true},
            {pubkey: walletUserDataAddr, isSigner: false, isWritable: true},
        ];

        console.log("initialize as", wallet.publicKey.toString(),
                    "for", usernameWalletAddr.toString(), "/", walletUserDataAddr.toString());

        let ixn = new w3.TransactionInstruction({
            keys: keys,
            programId: PROGRAM_ID,
            data: data,
        });

        let txn = new w3.Transaction().add(ixn);

        let res = await w3.sendAndConfirmTransaction(
            conn,
            txn,
            [wallet],
            {commitment: COMMITMENT, preflightCommitment: COMMITMENT, skipPreflight: SKIP_PREFLIGHT},
        );

        console.log("initialize res:", res);
        return res;
    },

    // {"CreateUser": {"username": STRING}}
    createUser: async (conn, wallet, username) => {
        // XXX idk how js is supposed to handle errors like this
        if(!username.match(username_regex)) {
            console.log("bad username:", username);
            return;
        }

        let data = Buffer.from(`{"CreateUser": {"username": "${username}"}}`, "utf8");
        let userAccount = new w3.Account();

        let keys = [
            {pubkey: wallet.publicKey, isSigner: true, isWritable: true},
            {pubkey: w3.SystemProgram.programId, isSigner: false, isWritable: false},
            {pubkey: w3.SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false},
            {pubkey: w3.SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false},
            {pubkey: usernameWalletAddr, isSigner: false, isWritable: true},
            {pubkey: walletUserDataAddr, isSigner: false, isWritable: true},
            {pubkey: userAccount.publicKey, isSigner: true, isWritable: true},
        ];

        let ixn = new w3.TransactionInstruction({
            keys: keys,
            programId: PROGRAM_ID,
            data: data,
        });

        let txn = new w3.Transaction().add(ixn);

        let res = await w3.sendAndConfirmTransaction(
            conn,
            txn,
            [wallet, userAccount],
            {commitment: "processed", preflightCommitment: "processed", skipPreflight: SKIP_PREFLIGHT},
        );

        console.log("create user res:", res);
        return res;
    },

};

(async () => {
    // init these globals
    usernameWalletAddr = (await w3.PublicKey.findProgramAddress([Buffer.from("USERNAME_WALLETS")], PROGRAM_ID))[0];
    walletUserDataAddr = (await w3.PublicKey.findProgramAddress([Buffer.from("WALLET_USERDATA")], PROGRAM_ID))[0];

    console.log("establishing connection");
    let conn = main.connect(NETWORK);

    console.log("loading wallet");
    let wallet = await main.wallet(conn);

    // maps from username to pubkey and pubkey to userdata address
    console.log("loading data");
    let userWallets = await get.struct(conn, usernameWalletAddr);
    let walletUsers = await get.struct(conn, walletUserDataAddr);

    // get userdata if it exists
    let userDataAddr = walletUsers[wallet.publicKey.toString()];
    //console.log("userdataaddr:", new w3.PublicKey(userDataAddr).toString());
    let user = userDataAddr ? await get.struct(conn, new w3.PublicKey(userDataAddr)) : null;
    //console.log("user:", user);

    //console.log("initializing chain storage");
    //await post.initialize(conn, wallet);

    console.log("creating user");
    await post.createUser(conn, wallet, "not_hana");

    return 0;
})();
