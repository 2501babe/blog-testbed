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
      <button @click="connectProvider()">connect</button>
    </span>
  </div>

  <h1 class="cozy center">navy</h1>

  <!-- tink ima make this a sidebar instead -->
  <!--div class="navbar">
    <a>home</a>
    <a>idk</a>
    <a>something</a>
  </div-->

  <div v-if="providerConnected" class="funbox">
    <div>left bar</div>

    <div>
      <textarea cols="100" rows="20" placeholder="you better write something good loser"/>
      <br/>
      <button>put it on the internet forever</button>
    </div>

    <div>right bar</div>
  </div>
  <div v-else>
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
    let vm = this;

    // capture the component so we can set ~reactive~ bits on it when page loads
    // FIXME this is kind of fucky but i cant figure out a better way
    // the ~reactive~ shit cant watch window variables so it all feels really brittle
    window.addEventListener("load", function() {
        vm.provider = window.solana;
        vm.providerConnected = window.solana.isConnected;
    });
  },
  data() {
    return {
      connectKey: 0,
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

        // FIXME if the user has to input their password this returns immediately
        // i could spinlock here but god would kill me and we cant detect if they reject the prompt anyway
        await vm.provider.connect();
        console.log("connected??", window.solana.isConnected);
        vm.providerConnected = true;
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

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>
.navbar {
    margin: 1em;
    display: flex;
    justify-content: space-around;
}

.funbox {
    margin: 1em;
    display: flex;
    justify-content: space-around;
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
