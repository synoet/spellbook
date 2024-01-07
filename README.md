# Spellbook

A simple fullstack rust application, built with axum and yew, that lets you semantically search for the command you're looking for. 


Commands are defined in this [gihub repo](https://github.com/synoet/spellbook-registry) , and when new commands are added / modified, a webhook is called that will generate embeddings for those commands and push them to a vector db. You can then search over those embeddings, finding a command semantically.


![CleanShot 2024-01-07 at 17 20 19](https://github.com/synoet/spellbook/assets/10552019/486d2c5d-eb6f-4dc3-bb73-c27261cce0e1)

This demo is quite rough but can be tried [here](https://www.spellbook.fly.dev)
