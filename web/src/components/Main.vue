<template>
  <div v-if="providerConnected" class="provider">
    <p>user: {{ prettyKey() }}</p>
  </div>
  <div v-else class="provider">
    <button @click="connectProvider()">connect</button>
  </div>
  <div>
    <h1>{{ msg }}</h1>
    <p>i love to blog</p>
    <p>{{ msg2 }}</p>
    <button v-on:click="reverseMsg">reverse</button>
    <p>{{ msg3 }}</p>
    <input v-model="msg3"/>
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
      msg2: "hey whats up",
      msg3: "ok cool"
    }
  },
  computed: {
  },
  props: {
    msg: String
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
