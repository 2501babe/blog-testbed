"use strict";

const w3 = require("@solana/web3.js");
const fs = require("fs").promises;
const path = require("path");

// constants
const KEYFILE = "testwallet.bin";
const NETWORK = "http://fortuna:8899";
const PROGRAM_ID = new w3.PublicKey("AbBrxmZKUJdn5ezmUUQSjefwojspSNSFwUDCHajg8H79");
const SKIP_PREFLIGHT = true;
const LAMPS = 1000000000;

const username_regex = /^[a-zA-Z][a-zA-Z0-9_]{0,31}$/;

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

// rust api
const api = {

    // {"Initialize": null}
    initialize: async (conn, wallet, userWallets, walletUserData) => {
        let data = Buffer.from('{"Initialize": null}', "utf8");

        let keys = [
            {pubkey: wallet.publicKey, isSigner: true, isWritable: true},
            {pubkey: w3.SystemProgram.programId, isSigner: false, isWritable: false},
            {pubkey: w3.SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false},
            {pubkey: userWallets, isSigner: false, isWritable: true},
            {pubkey: walletUserData, isSigner: false, isWritable: true},
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
            [wallet],
            {commitment: "processed", preflightCommitment: "processed", skipPreflight: SKIP_PREFLIGHT},
        );

        console.log("initialize res:", res);
        return res;
    },

    // {"CreateUser": {"username": STRING}}
    createUser: async (conn, wallet, userWallets, walletUserData, username) => {
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
            {pubkey: w3.SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false},
            {pubkey: userWallets, isSigner: false, isWritable: true},
            {pubkey: walletUserData, isSigner: false, isWritable: true},
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
    // debauched solweb3 devs use an async sha256 so this cant be a toplevel constant
    let userWallets = (await w3.PublicKey.findProgramAddress([Buffer.from("USERNAME_WALLETS")], PROGRAM_ID))[0];
    let walletUserData = (await w3.PublicKey.findProgramAddress([Buffer.from("WALLET_USERDATA")], PROGRAM_ID))[0];

    console.log("establishing connection");
    let conn = main.connect(NETWORK);

    console.log("loading wallet");
    let wallet = await main.wallet(conn);

    //console.log("initializing chain storage");
    //await api.initialize(conn, wallet, userWallets, walletUserData);

    console.log("creating user");
    await api.createUser(conn, wallet, userWallets, walletUserData, "hana");

    return 0;
})();
