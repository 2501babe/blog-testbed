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
    <div class="sidebar left">
      <a>home</a>
      <a>idk</a>
      <a>something</a>
    </div>

    <!-- main window ig just let ppl post here. maybe nav switches this area exclusively -->
    <div class="zone center">
      <input v-model="createPostForm.title" placeholder="title"/>
      <br/>
      <input v-model="createPostForm.uri" placeholder="uri"/>
      <br/>
      <textarea v-model="createPostForm.text" cols="100" rows="20" placeholder="you better write something good"/>
      <br/>
      <button @click="createPost()">put it on the internet forever</button>
    </div>

    <!-- info for the logged in user plus their posts -->
    <div class="sidebar right">
      <div v-if="userdata">
        <!-- handle, disply name, etc -->
        <div>
          <div style="font-size:1.5em">{{ userdata.display }}</div>
          <div style="opacity:50%">~{{ userdata.handle }}</div>
        </div>

        <!-- TODO posts -->
      </div>
      <div v-else>
        <table>
          <tr>
            <td align="right">display name</td>
            <td align="left"><input v-model="createUserForm.display"/></td>
          </tr>
          <tr>
            <td align="right">handle</td>
            <td align="left"><input v-model="createUserForm.handle"/></td>
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
const URI_REGEX = /^[a-zA-Z][a-zA-Z0-9_-]*$/;
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
    let providerLoop = setInterval(() => {
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
            clearInterval(providerLoop);
        }
    }, 100);

    // connect to selected network, load data immediately
    // and then set up a sentinal to continuously refresh as needed
    vm.connectChain();
    vm.loadData(true);
    setInterval(() => vm.loadData(), 2 * 60 * 1000);
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
      createPostForm: {title: "", uru: "", text: ""},
    }
  },
  watch: {
    // reconnect and force refresh our data when network changes
    selectedNetwork(curr, prev) {
        let vm = this;

        if(prev != curr) {
            vm.connectChain();
            vm.loadData(true);
        }
    },
    async walletAddress() {
        let vm = this;

        let userdata_addr = vm.walletAddress && vm.walletUserdata[vm.walletAddress];
        if(userdata_addr) {
            vm.userdata = await vm.getStruct(new w3.PublicKey(userdata_addr));
        }
    },
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
    connectChain() {
        let vm = this;

        // in theory letting an existing connection scope out should disconnect it
        // regardles sthe object has no explicit disconnect
        vm.connection = new w3.Connection(NETWORKS[vm.selectedNetwork]);
        console.log("connected to", vm.selectedNetwork);
    },
    // load the hashmaps of user account info
    // etag is a counter incremented onchain whenever the structs change
    async loadData(force = false) {
        let vm = this;

        let etag = await vm.getEtag((await ETAG_PROMISE)[0]);

        // if forced or data has changed or we havent fetched yet
        if(force || etag != vm.etag || etag == 0) {
            vm.handleWallets = await vm.getStruct((await USRWAL_PROMISE)[0]);
            vm.walletUserdata = await vm.getStruct((await WALUSR_PROMISE)[0]);
            vm.etag = etag;

            // load userdata also if we have a user
            let userdata_addr = vm.walletAddress && vm.walletUserdata[vm.walletAddress];
            if(userdata_addr) {
                vm.userdata = await vm.getStruct(new w3.PublicKey(userdata_addr));
            }
        }
    },

    // API ZONE this shit should be a component or something but 
    // i dont know how to pass data between them easily lolz

    // read etag and retain as number
    // this is supposed to be a u64 on server but js numbers are stupidpilled
    // but it doesnt matter since we only care that it changes
    async getEtag(addr) {
        let vm = this;

        let acct = await vm.connection.getAccountInfo(addr, COMMITMENT);
        let dv = acct && new DataView(acct.data.buffer);
        return dv ? dv.getUint32(4) : 0;
    },
    // read account data and parse into json
    async getStruct(addr) {
        let vm = this;

        let acct = await vm.connection.getAccountInfo(addr, COMMITMENT);
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
        await vm.loadData();

        return signature;
    },
    async createPost() {
        let vm = this;
        let form = vm.createUserForm;

        if(!form.uri.match(URI_REGEX)) {
            console.log("bad uri:", uri);
            return;
        }

        let buf1 = Buffer.from(JSON.stringify({CreatePostdata: {title: title, uri: uri}}));
        let buf2 = Buffer.from(JSON.stringify({CreatePost: {text: text}}));
        let postAccount = new w3.Account();
        let etagKey = (await ETAG_PROMISE)[0];
        let userdataAddr = vm.walletAddress && vm.walletUserdata[vm.walletAddress];

        let sharedKeys = [
            {pubkey: vm.provider.publicKey, isSigner: true, isWritable: true},
            {pubkey: w3.SystemProgram.programId, isSigner: false, isWritable: false},
            {pubkey: w3.SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false},
            {pubkey: w3.SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false},
            {pubkey: etagKey, isSigner: false, isWritable: true},
        ];
        let udKey = {pubkey: new w3.PublicKey(userdataAddr), isSigner: false, isWritable: true};
        let postKey = {pubkey: postAccount.publicKey, isSigner: true, isWritable: true};

        // XXX TODO FIXME uhh take the rest of the stuff from main in here
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
