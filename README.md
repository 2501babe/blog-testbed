navy
---

![screenshot](https://github.com/2501babe/blog-testbed/blob/master/image.png)

ok so as you can see this is not terribly exciting i started learning solana for this hackathon lol\
you can connect your wallet and claim a username, which gets associated to you on chain\
and then write and upload posts, which i have working in `main.js` but didnt get into the dapp in time\
i ran into a lot of problems last minute with the compute budget and transaction size limit\
soo lots of work still to do

anyway the idea is to give users a way to share thoughts and ideas tied to their wallet\
web3 is basically about having accounts built into your browser that constitute your identity\
rather than the site controlling who you are, you do, interacting with the blockchain directly\
low storage cost on solana means text can easily be stored directly on the chain\
this means posts can be stored permissionlessly and permanently\

storage and display are effectively decoupled, and moderation moves to the endpoint\
that is, a neutral service stores text and metadata onchain, and various frontends are free to serve it\
and of course any user can query the data themself from an rpc endpoint and read it in the raw

soo a roadmap would look something like
* make posting work on the dapp (lol)
* make pages that display posts per user and surface posts from unknown users
* implement post chunking to get around the transaction size limit, like how program deployment works
* make the site prettier (lol)

and then feature ideas include letting users post without a username\
(as it stands, userdata is unecessarily intertwined with postdata)\
id like to add categories and tags that you can attach to posts and which are browsable by readers\
another cool idea i had was a way to irrevocably lock posts, where an author can effectively remove their own edit rights\
partly as a flex but partly as a way to make it impossible for someone to coerce an edit

umm thats it! i know its not much but i hope you like\
my brain is small but my heart is big and im glad to have had the chance to (partly) build this
