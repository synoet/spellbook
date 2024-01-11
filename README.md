# Spellbook
A simple fullstack rust application, built with axum and yew, that lets you semantically search for the command you're looking for. 

Commands are defined in this [gihub repo](https://github.com/synoet/spellbook-registry) , and when new commands are added / modified, a webhook is called that will generate embeddings for those commands and push them to a vector db. You can then search over those embeddings, finding a command semantically.

https://github.com/synoet/spellbook/assets/10552019/ec9a62cd-1ada-4a2b-9449-1666e4244556

This demo is quite rough but can be tried [here](https://spellbook.fly.dev/)





