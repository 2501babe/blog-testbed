<template>
  <div>
    <h1>{{ msg }}</h1>
    <p>i love to blog</p>
  </div>
</template>

<script>
export default {
  name: 'Main',
  props: {
    msg: String
  }
}
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>
</style>
