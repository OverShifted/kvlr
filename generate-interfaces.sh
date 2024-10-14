#!/bin/sh

pushd bnzn
python main.py test.bnzn -oc ../kvlr-client/src/client.rs -os ../kvlr-server/src/server_trait.rs
popd
