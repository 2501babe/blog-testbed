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
      user: {{ prettyKey() }}
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

    <div>
      <textarea cols="100" rows="20" placeholder="you better write something good loser"/>
      <br/>
      <button>put it on the internet forever</button>
    </div>

    <div>right bar</div>
  </div>
  <div v-else class="center">
    <p>do you love to blog?</p>
    <p>do you know wtf "solana" is??</p>
    <p>wow youre so smart howd you find this site!!</p>
  </div>

</template>

<script>
//import * as w3 from "../node_modules/@solana/web3.js";

export default {
  name: 'Main',
  mounted() {
    let vm = window.main = this;

    /// this is evil but i havent found a better way
    // even waiting until the page loads doesnt guarantee window.solana will be set
    // and there is no good way to monitor isConnected since connect resolves before actually connecting
    setInterval(() => {
        if(!vm.provider) {
            vm.provider = window.solana;
        }

        if(window.solana && vm.providerConnected != window.solana.isConnected) {
            vm.providerConnected = window.solana.isConnected;
        }
    }, 200);
  },
  data() {
    return {
      provider: null,
      providerConnected: false,
      selectedNetwork: "fortuna",
    }
  },
  computed: {
  },
  props: {
  },
  methods: {
    async connectProvider() {
        let vm = this;

        if(!vm.provider) {
            alert("no wallet provider found");
            return;
        }

        if(!vm.provider.isPhantom) {
            alert("i only support phantom because im lazy");
            return;
        }

        await vm.provider.connect();
    },
    reverseMsg() {
      this.msg2 = this.msg2.split("").reverse().join("");
    },
    prettyKey() {
        let vm = this;
        let pubkey = vm.provider.publicKey.toString();
        return pubkey.substring(0, 4) + "..." + pubkey.substring(pubkey.length - 4);
    }
  }
}
</script>

<style scoped>
.funbox {
    margin: 1em;
    display: flex;
    justify-content: space-evenly;
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
