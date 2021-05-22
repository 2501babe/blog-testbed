"use strict";

const w3 = require("@solana/web3.js");
const fs = require("fs").promises;
const path = require("path");

// constants
const KEYFILE = "testwallet.bin";
const NETWORK = "http://fortuna:8899";
const PROGRAM_ID = new w3.PublicKey("AbBrxmZKUJdn5ezmUUQSjefwojspSNSFwUDCHajg8H79");
const LAMPS = 1000000000;

// basic program shit
const main = {

    connect: (network) => {
        let conn = new w3.Connection(network);
        return conn;
    },

    getWallet: async (conn) => {
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

    ping: async (conn, wallet) => {
        let data = Buffer.from("ping!", "utf8");

        let ixn = new w3.TransactionInstruction({
            keys: [{pubkey: wallet.publicKey, isSigner: true, isWritable: true}],
            programId: PROGRAM_ID,
            data: data,
        });

        let txn = new w3.Transaction().add(ixn);

        let res = await w3.sendAndConfirmTransaction(
            conn,
            txn,
            [wallet],
            {commitment: "processed", preflightCommitment: "processed", skipPreflight: false},
        );

        console.log("res:", res);
        return res;
    },

};

(async () => {
    console.log("establishing connection");
    let conn = main.connect(NETWORK);

    console.log("loading wallet");
    let wallet = await main.getWallet(conn);

    console.log("pinging chain");
    await api.ping(conn, wallet);

    return 0;
})();
