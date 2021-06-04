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
    <div class="sidebar">
      <a>home</a>
      <a>idk</a>
      <a>something</a>
    </div>

    <div class="zone">
      <textarea cols="100" rows="20" placeholder="you better write something good loser"/>
      <br/>
      <button>put it on the internet forever</button>
    </div>

    <div class="sidebar">
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

const PROGRAM_ID = new w3.PublicKey("AbBrxmZKUJdn5ezmUUQSjefwojspSNSFwUDCHajg8H79");

// XXX i should polyfill my own findProgramAddress replacement
// its async because they insist of using an async shasum for no reason
const USRWAL_PROMISE = w3.PublicKey.findProgramAddress([Buffer.from("USERNAME_WALLETS")], PROGRAM_ID);
const WALUSR_PROMISE = w3.PublicKey.findProgramAddress([Buffer.from("WALLET_USERDATA")], PROGRAM_ID);

/*
const COMMITMENT = "processed";
const SKIP_PREFLIGHT = true;

const USERNAME_REGEX = /^[a-zA-Z][a-zA-Z0-9_]{0,31}$/;
*/

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
      usernameWallets: null,
      walletUserData: null,
      userData: null,
    }
  },
  computed: {
    // print first and last four of a pubkey with ellipsis
    prettyKey() {
        let vm = this;

        return vm.walletAddress.substring(0, 4) + "..." + vm.walletAddress.substring(vm.walletAddress.length - 4);
    },
    // data block for the currently connected user
    userdata() {
        let vm = this;
        let user = vm.walletUserData && vm.walletUserData[vm.walletAddress];

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

        vm.usernameWallets = await vm.getStruct((await USRWAL_PROMISE)[0]);
        vm.walletUserData = await vm.getStruct((await WALUSR_PROMISE)[0]);
    },

    // API ZONE this shit should be a component or something but 
    // i dont know how to pass data between them easily lolz

    // read account data and parse into json
    async getStruct(addr) {
        let vm = this;

        let acct = await vm.connection.getAccountInfo(addr);
        let str = acct ? acct.data.toString().split("\0").shift() : "";
        return str.length > 0 ? JSON.parse(str) : {};
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

.right {
    align: right;
    text-align: right;
}

.center {
    align: center;
    text-align: center;
}

.provider {
    align: right;
    text-align: right;
    margin: 1em;
}
</style>
