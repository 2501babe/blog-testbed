<template>

  <div class="cozy right">
    <select v-model="selectedNetwork">
      <option>mainnet</option>
      <option>testnet</option>
      <option>devnet</option>
      <option>localhost</option>
      <option>fortuna</option>
    </select>

    <span v-if="providerConnected">
      user: {{ prettyKey }}
    </span>
    <span v-else>
      <button :disabled="!provider" @click="connectProvider()">connect</button>
    </span>
  </div>

  <h1 class="cozy center">navy</h1>

  <div v-if="providerConnected" class="funbox">
    <!-- nav and shit, idk what goes here yet -->
    <div class="sidebar">
      <a>home</a>
      <a>idk</a>
      <a>something</a>
    </div>

    <!-- main window ig just let ppl post here. maybe nav switches this area exclusively -->
    <div class="zone">
      <textarea cols="100" rows="20" placeholder="you better write something good loser"/>
      <br/>
      <button>put it on the internet forever</button>
    </div>

    <!-- info for the logged in user plus their posts -->
    <div class="sidebar">
      <div v-if="userdata">
        display user here
      </div>
      <div v-else>
        <table>
          <tr>
            <td align="right">handle</td>
            <td align="left"><input v-model="createUserForm.handle"/></td>
          </tr>
          <tr>
            <td align="right">display name</td>
            <td align="left"><input v-model="createUserForm.display"/></td>
          </tr>
        </table>
        <div @click="createUser()" class="center"><button>register</button></div>
      </div>
    </div>
  </div>
  <div v-else class="center">
    <p>do you love to blog?</p>
    <p>do you know wtf "solana" is??</p>
    <p>wow youre so smart howd you find this site!!</p>
  </div>

</template>

<script>
import * as w3 from "../../node_modules/@solana/web3.js";

const PROGRAM_ID = new w3.PublicKey("EMJjWij5oLb2usWknxmcpzm6bsgktLxEHPMsafiHDX7e");

// XXX i should polyfill my own findProgramAddress replacement
// its async because they insist of using an async shasum for no reason
const ETAG_PROMISE = w3.PublicKey.findProgramAddress([Buffer.from("ETAG")], PROGRAM_ID);
const USRWAL_PROMISE = w3.PublicKey.findProgramAddress([Buffer.from("HANDLE_WALLETS")], PROGRAM_ID);
const WALUSR_PROMISE = w3.PublicKey.findProgramAddress([Buffer.from("WALLET_USERDATA")], PROGRAM_ID);

const COMMITMENT = "processed";
const SKIP_PREFLIGHT = false;

const HANDLE_REGEX = /^[a-zA-Z][a-zA-Z0-9_]{0,23}$/;
const DISPLAY_LENGTH = 32;

const NETWORKS = {
    "mainnet": "https://solana-api.projectserum.com",
    "testnet": "https://api.testnet.solana.com",
    "devnet": "https://api.devnet.solana.com",
    "localhost": "http://127.0.0.1:8899",
    "fortuna": "http://fortuna:8899",
};

export default {
  name: 'Main',
  mounted() {
    let vm = window.main = this;

    // loop until the wallet shows up on window, this is slower than vue
    let ivl = setInterval(() => {
        if(window.solana && !vm.provider) {
            vm.provider = window.solana;
            vm.provider.on("connect", () => {
                vm.providerConnected = true;
                vm.walletAddress = vm.provider.publicKey.toString();
            });
            vm.provider.on("disconnect", () => {
                vm.providerConnected = false;
                vm.walletAddress = null;
            });
            clearInterval(ivl);
        }
    }, 100);

    // fetching data can be done independent of wallet
    // XXX this needs to react to swapping th dropdown
    vm.connectChain();
  },
  data() {
    return {
      // wallet stuff
      provider: null,
      providerConnected: false,
      walletAddress: null,
      // chain connection
      selectedNetwork: "fortuna",
      connection: null,
      // lookup tables. theres no way to query data onchain
      // so we just ahve to load the whole things into memory lolz
      etag: 0,
      handleWallets: null,
      walletUserdata: null,
      userdata: null,
      //input form data zone
      // XXX this shit should def be a separate component lol
      createUserForm: {handle: "", display: ""},
    }
  },
  computed: {
    // print first and last four of a pubkey with ellipsis
    prettyKey() {
        let vm = this;

        return vm.walletAddress.substring(0, 4) + "..." + vm.walletAddress.substring(vm.walletAddress.length - 4);
    },
    // data block for the currently connected user
    userdataAddress() {
        let vm = this;
        let user = vm.walletUserdata && vm.walletUserdata[vm.walletAddress];

        return user || null;
    },
  },
  props: {
  },
  methods: {
    // connect wallet
    connectProvider() {
        let vm = this;

        if(!vm.provider) {
            alert("no wallet provider found");
            return;
        }

        if(!vm.provider.isPhantom) {
            alert("i only support phantom because im lazy");
            return;
        }

        vm.provider.connect();
    },
    // connect to the selected chain
    async connectChain() {
        let vm = this;

        vm.connection = new w3.Connection(NETWORKS[vm.selectedNetwork]);
        console.log("connected to", vm.selectedNetwork);

        await vm.loadData();
    },
    // load the hashmaps of user account info
    async loadData() {
        let vm = this;

        vm.etag = await vm.getEtag((await ETAG_PROMISE)[0]);
        vm.handleWallets = await vm.getStruct((await USRWAL_PROMISE)[0]);
        vm.walletUserdata = await vm.getStruct((await WALUSR_PROMISE)[0]);
    },

    // API ZONE this shit should be a component or something but 
    // i dont know how to pass data between them easily lolz

    // read etag and retain as number
    // this is supposed to be a u64 on server but js numbers are stupidpilled
    // but it doesnt matter since we only care that it changes
    async getEtag(addr) {
        let vm = this;

        let acct = await vm.connection.getAccountInfo(addr);
        let dv = acct && new DataView(acct.data.buffer);
        return dv ? dv.getUint32(4) : -1;
    },
    // read account data and parse into json
    async getStruct(addr) {
        let vm = this;

        let acct = await vm.connection.getAccountInfo(addr);
        let str = acct ? acct.data.toString().split("\0").shift() : "";
        return str.length > 0 ? JSON.parse(str) : {};
    },
    // {"CreateUser": {"handle": STRING, "display": STRING}}
    async createUser() {
        let vm = this;
        let form = vm.createUserForm;

        if(!form.handle.match(HANDLE_REGEX)) {
            alert("handles are 1-24 alphanum/underscore, not starting with underscore");
            return;
        }

        // XXX im like, 80% sure js length is byes not codepoints lol
        if(form.display.length > DISPLAY_LENGTH) {
            alert("display names are at most 32 characters");
            return;
        }

        if(vm.userdata) {
            alert("user exists pls use updateUser");
            return;
        }

        if(!form.display) {
            form.display = "~" + form.handle;
        }

        let data = Buffer.from(JSON.stringify({CreateUser: form}));
        let userAccount = new w3.Account();
        let etagKey = (await ETAG_PROMISE)[0];
        let usrwalKey = (await USRWAL_PROMISE)[0];
        let walusrKey = (await WALUSR_PROMISE)[0];

        let keys = [
            {pubkey: vm.provider.publicKey, isSigner: true, isWritable: true},
            {pubkey: w3.SystemProgram.programId, isSigner: false, isWritable: false},
            {pubkey: w3.SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false},
            {pubkey: w3.SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false},
            {pubkey: etagKey, isSigner: false, isWritable: true},
            {pubkey: usrwalKey, isSigner: false, isWritable: true},
            {pubkey: walusrKey, isSigner: false, isWritable: true},
            {pubkey: userAccount.publicKey, isSigner: true, isWritable: true},
        ];

        let ixn = new w3.TransactionInstruction({
            keys: keys,
            programId: PROGRAM_ID,
            data: data,
        });

        let txn = new w3.Transaction().add(ixn);
        txn.feePayer = vm.provider.publicKey;
        txn.recentBlockhash = (await vm.connection.getRecentBlockhash(COMMITMENT)).blockhash;
        txn.partialSign(userAccount);
        await vm.provider.signTransaction(txn);

        let signature = await w3.sendAndConfirmRawTransaction(
            vm.connection,
            txn.serialize(),
            {commitment: COMMITMENT, preflightCommitment: COMMITMENT, skipPreflight: SKIP_PREFLIGHT},
        );

        console.log("CREATE USER:", signature);
        return signature;
    },
  }
}
</script>

<style scoped>
.funbox {
    margin-top: 4em;
    display: flex;
    justify-content: space-evenly;
}

.zone {
}

.sidebar {
    display: flex;
    flex-direction: column;
}

.cozy  {
    margin: 0.25em;
}

.left {
    text-align: left;
}

.right {
    text-align: right;
}

.center {
    text-align: center;
}

.provider {
    text-align: right;
    margin: 1em;
}
</style>
