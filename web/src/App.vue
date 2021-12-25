<template>
  <Main/>
</template>

<script>
import Main from "./components/Main.vue";
//import * as w3 from "../node_modules/@solana/web3.js";

/* XXX OK cheatsheet time
   thing passed into createApp is the root component, same as any other
   in the actual index.html the "app" id is our container since we pass in "#app" to mount
   they use webpack which idk how that shit works but you can do like
   `import * as lol "../node_modules/a/b.js"`. worry how to actually ship it later lol
   async and await seem to work with no special bullshit

   wow its components
   - vue doesnt do string templating, the standard reuse element is the component
   - parents pass data to children via prop attributes
     children can send events with $emit, parent can listen with v-on
   - you can use v-model with this shit but i skipped reading it
   - theres a <slot> tag to pass "content" (like uh, html?)
   - <component :is="something"> to dynamically swap, eg tab interface
   - table and list shit you need to do like <li v-is="'something'">
     yes the quotes have to be like that

   shit that goes in instance properties
   - data: function that returns an object, declares two-way bindings that go on the dom
   - created: function run after app init. theres others but this one prolly most useful
   - methods: object of functions eg for onclicks. `this` bound to component
     calling these in templates is less preferable than using computed properties
     do debouncing by putting eg `clickme` in `methods` then doing like
     `this.dbclick = _.debounce(this.clickme, 500)` in created hook
   - computed: object of functions. these are usuable just like shit in data
     example they give is, data has a list of books the user can fuck with
     computed has a function that returns some message depending on if the book list is empty
     vue is aware of data dependencies so these update when the things used in them do
     uhh this actually is identical functionally to putting it in methods
     the only difference is vue caches the result
   - watch: more complicated shit, eg call out to an api based on some condition
   - props: array of var names. can also use object, values are types to cast to
     these go on the child, indicating variables it accepts
     pass them in as attrs directly from parent. computed shit can go in the value
     eg, child: `props: ["lol"]`, parent: <child lol="1">

   shit that goes in html attributes
   - v-html: dump raw html into the relevant block
   - v-bind:ATTR: put variable in declared attribute. : shorthand
     you can pass class an object like { classname: bound-var-to-toggle }
     can also pass style an object of keys and values for css
   - v-on:EVT.MOD: onclick events and shit. @ shorthand
     the actual dom event is exposed as $event in the attr to be passed as arg
     modifiers include: stop prevent capture self once passive
     these are webshit lore ig, eg form @submit.prevent blackholes a refresh
     key names are also valid modifers for keyboard events, eg @keyup.alt.enter
   - v-model: two-way data bindings for interactive shit
     lots of examples here i always forget how these work https://v3.vuejs.org/guide/forms.html
     theres a .trim modifier to excise whitespace how helpful
   - v-if: insert an html block or dont based on bool. also v-else, v-else-if
     the elses should be full blocks of their own immediately following
   - v-show: toggles the display property, unlike v-if it always goes in the dom
   - v-for: list display, use like <li v-for="thing in things"> {{ thing.prop }} </li>
     soo inside a ul tag with a list of six things generates six li blocks
     can also do `(thing, index) in things`. also works with object. val first key second
     theres a lot more bullshit this can do just recheck docs if needed
*/

// XXX OK COOL how am i structuring this what do i nede
// * user goes to page and sees a connect button and some splash shit
// * clicky connect, do the phantom setup dance. yay window.solana works now
// * maybe just one main page for now. textarea on left for making a post
//   list of existing posts on right. some place to register user info
// XXX ok that shit worked fine. vue is actually pretty easy
// next up i need to port my api code stuff over
// have a network dropdown or something, fetch my hashmaps, fill out user
// what does page actually look like
// nav on left post box in center user info and own post list on right?

// XXX oki aaa what next tomorrow. set up web3 in the app
// put user and post info in the right sidebar
// impl createpost
// then i need to think what else i need...
// * make pubkeys strings
// * add tags, category, notional timestamp, summary
// and then... what other pages do we want. maybe some kind of discover page
// oh blog stream page for every author obv lol. good enough for now

// XXX NEXT THINGS
// * periodically refresh hashmaps, maybe have a "last updated" sentinal account
// * watcher to fill userdata from hashmap
// * create post impl

// XXX OKI DEADLINE IS TODAY
// what are mu must haves
// * store th epost make th postdata
// * list posts on the right bar
// * click a post to have it come up in the center
// NICE TO HAVES
// * new post, discovery page links on the left bar
// * discovery page listing others users posts

export default {
  name: 'App',
  components: {
    Main
  },
}
</script>

<style>
</style>
