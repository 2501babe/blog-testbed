const w3 = require("@solana/web3.js");
const fs = require("fs").promises;
const path = require("path");

// constants
const KEYFILE = "testwallet.bin";
const NETWORK = "http://fortuna:8899";
const LAMPS = 1000000000;

// basic program shit
const main = {

    connect: (network) => {
        const conn = new w3.Connection(network);
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

const conn = main.connect(NETWORK);
main.getWallet(conn);
