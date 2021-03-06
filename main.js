"use strict";

const w3 = require("@solana/web3.js");
const fs = require("fs").promises;
const path = require("path");

// constants
const LAMPS = 1000000000;
const KEYFILE = "testwallet.bin";
const NETWORK = "http://fortuna:8899";
const PROGRAM_ID = new w3.PublicKey("EMJjWij5oLb2usWknxmcpzm6bsgktLxEHPMsafiHDX7e");

// for debug these should be processed (faster) and true (actually test the server)
// but in prod we prolly want finalized and false
const COMMITMENT = "processed";
const SKIP_PREFLIGHT = true;

const handle_regex = /^[a-zA-Z0-9][a-zA-Z0-9_]{0,23}$/;
const uri_regex = /^[a-zA-Z][a-zA-Z0-9_-]*$/;

// debauched solweb3 devs use an async sha256 so these cant be toplevel constants
var etagAddr;
var handleWalletAddr;
var walletUserdataAddr;

// ok this one im just lazy
var userdataAddr;

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
            {pubkey: etagAddr, isSigner: false, isWritable: true},
            {pubkey: handleWalletAddr, isSigner: false, isWritable: true},
            {pubkey: walletUserdataAddr, isSigner: false, isWritable: true},
        ];

        console.log("initialize as", wallet.publicKey.toString(),
                    "for", etagAddr.toString(), "/", handleWalletAddr.toString(), "/", walletUserdataAddr.toString());

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

    // {"CreateUser": {"handle": STRING, "display": STRING}}
    createUser: async (conn, wallet, handle, display) => {
        // XXX idk how js is supposed to handle errors like this
        if(!handle.match(handle_regex)) {
            console.log("bad handle:", handle);
            return;
        }

        let data = Buffer.from(JSON.stringify({CreateUser: {handle: handle, display: display}}));
        let userAccount = new w3.Account();

        let keys = [
            {pubkey: wallet.publicKey, isSigner: true, isWritable: true},
            {pubkey: w3.SystemProgram.programId, isSigner: false, isWritable: false},
            {pubkey: w3.SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false},
            {pubkey: w3.SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false},
            {pubkey: etagAddr, isSigner: false, isWritable: true},
            {pubkey: handleWalletAddr, isSigner: false, isWritable: true},
            {pubkey: walletUserdataAddr, isSigner: false, isWritable: true},
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
        userdataAddr = userAccount.publicKey;
        return res;
    },

    // {"CreatePost": {"title": STRING, "uri": STRING, "text": STRING}}
    createPost: async (conn, wallet, title, uri, text) => {
        if(!uri.match(uri_regex)) {
            console.log("bad uri:", uri);
            return;
        }

        let buf1 = Buffer.from(JSON.stringify({CreatePostdata: {title: title, uri: uri}}));
        let buf2 = Buffer.from(JSON.stringify({CreatePost: {text: text}}));

        let postAccount = new w3.Account();

        let sharedKeys = [
            {pubkey: wallet.publicKey, isSigner: true, isWritable: true},
            {pubkey: w3.SystemProgram.programId, isSigner: false, isWritable: false},
            {pubkey: w3.SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false},
            {pubkey: w3.SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false},
            {pubkey: etagAddr, isSigner: false, isWritable: true},
        ];
        let udKey = {pubkey: userdataAddr, isSigner: false, isWritable: true};
        let postKey = {pubkey: postAccount.publicKey, isSigner: true, isWritable: true};

        console.log("HANA userdata addr:", userdataAddr.toString());

        let ixn1 = new w3.TransactionInstruction({
            keys: sharedKeys.concat([udKey, postKey]),
            programId: PROGRAM_ID,
            data: buf1,
        });
        let ixn2 = new w3.TransactionInstruction({
            keys: sharedKeys.concat([postKey]),
            programId: PROGRAM_ID,
            data: buf2,
        });

        let txn = new w3.Transaction().add(ixn1).add(ixn2);

        let res = await w3.sendAndConfirmTransaction(
            conn,
            txn,
            [wallet, postAccount],
            {commitment: "processed", preflightCommitment: "processed", skipPreflight: SKIP_PREFLIGHT},
        );

        console.log("create post res:", res);
        return res;
    },

};

(async () => {
    // init these globals
    etagAddr = (await w3.PublicKey.findProgramAddress([Buffer.from("ETAG")], PROGRAM_ID))[0];
    handleWalletAddr = (await w3.PublicKey.findProgramAddress([Buffer.from("HANDLE_WALLETS")], PROGRAM_ID))[0];
    walletUserdataAddr = (await w3.PublicKey.findProgramAddress([Buffer.from("WALLET_USERDATA")], PROGRAM_ID))[0];

    console.log("establishing connection");
    let conn = main.connect(NETWORK);

    console.log("loading wallet");
    let wallet = await main.wallet(conn);

    // maps from handle to pubkey and pubkey to userdata address
    console.log("loading data");
    let userWallets = await get.struct(conn, handleWalletAddr);
    let walletUsers = await get.struct(conn, walletUserdataAddr);

    // get userdata if it exists
    userdataAddr = walletUsers[wallet.publicKey.toString()];
    //console.log("userdataaddr:", new w3.PublicKey(userdataAddr).toString());
    let user = userdataAddr ? await get.struct(conn, new w3.PublicKey(userdataAddr)) : null;
    //console.log("user:", user);

    console.log("initializing chain storage");
    await post.initialize(conn, wallet);

    console.log("creating user");
    await post.createUser(conn, wallet, "not_hana", "who could this be");

    console.log("creating post");
    await post.createPost(conn, wallet, "top 100 sweets i like", "top-100-sweets", "ummmm cookie dough and dark chocolate and cake and...\n\neclairs and cannolis and donuts and muffins andddd\n\nok im tired now ima take a nap");

    return 0;
})();
