<template>
  <div v-if="providerConnected" class="provider">
    <p>user: {{ prettyKey() }}</p>
  </div>
  <div v-else class="provider">
    <button @click="connectProvider()">connect</button>
  </div>
  <h1>navy</h1>
  <div v-if="providerConnected">
    <textarea cols="100" rows="20" placeholder="you better write something good loser"/>
    <br/>
    <button>put it on the internet forever</button>
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
    window.addEventListener("load", function() {
        vm.provider = window.solana;
    });
  },
  data() {
    return {
      connectKey: 0,
      provider: null,
      providerConnected: false,
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
.provider {
    align: right;
    text-align: right;
    margin: 1em;
}
</style>
