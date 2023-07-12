
# Setting Up The Bootnode

To help students bootstrap a network, and to allow them to get experience as a user before they run their own nodes, it is important that the instructor have access to a bootnode hosted somewhere accessible to all students.
Some instructors may choose to host this node themselves, others may prefer to have PBA run some infrastructure.
In either case, these notes walk through the main steps of configuring the bootnode and its server.

## Requirements

We need a node that students can connect to with their own nodes over the P2P network, and also with a user interface over the websocket endpoint.
For the latter, it is very helpful to have the websocket behind an ssl endpoint (wss://) so that UIs hosted on https sites can still connect.

## System Overview

The basic pieces you need and the tool these notes recommend to achieve it.

VPS - digital ocean
OS - Ubuntu 22.04
SSL cert - Let's encrypt
Reverse proxy - nginx
Academy PoW node - copied in binary

The ssl cert and reverse proxy are all so that students can use UIs like Apps from the hosted website without warnings or errors.
If this is not important to you, you can save some trouble by skipping the SSL stuff.

## Base System Setup

Create a new VPS starting with Ubuntu 22.04 with SSH permissions.
I use a digital ocean droplet for these purposes.

Many of the following commands need to be run as root, so it is convenient to start a persistent sudo session with `sudo -i`.

## Nginx Webserver and Reverse Proxy

We will use Nginx as a reverse proxy and to host a simple web page that points users to relevant sites like the UI and the github repo.

We use `apt` start by installing it.

```bash
apt install nginx
```

Confirm the server is working by going directly to your IP in the browser. Should see the nginx test page. The file you just viewed lives at `/var/www/html/index.nginx-debian.html`. This file is only used when a regular `index.html` file is not present. Let's create our own index file.

```bash
cd /var/www/html
vim index.html
```

Then paste in this content
```html
<!DOCTYPE html>
<html>
	<head>
		<title>Denver Substrate Testnet</title>
		<style>
			body {
				width: 35em;
				margin: 0 auto;
			}
		</style>
	</head>
	<body>
		<h1>Denver Substrate Testnet</h1>
		<p>Thanks for joining our Substrate testnet!</p>

		<ul>
			<li>Follow along with <a href="https://substrate.dev/substrate-beginner-workshop/#/0/">The Workshop</a></li>
			<li>Clone the <a href="https://github.com/substrate-developer-hub/substrate-node-template">Substrate Node Template</a>.</li>
			<li>Download the <a href="spec.json">Chain Specification</a>.</li>
			<li>Download the <a href="alice.json">Prefunded Alice Key</a>.</li>
			<li>Connect to <a href="https://polkadot.js.org/apps?rpc=wss://denver.bootnodes.net/alice">Alice's Node</a></li>
			<li>Connect to <a href="https://polkadot.js.org/apps?rpc=wss://denver.bootnodes.net/bob">Bob's Node</a></li>
		</ul>

	</body>
</html>

```

We'll create those linked files later. Confirm the new webpage loads.

## Adding SSL
We need a domain in order to use SSL. Register it and setup dns so it points to your server. This process varies a lot by registrar. When your domain loads the webpage we just created, you may proceed to setup SSL.

For setting up subdomains like sfbw.bootnodes.net just use an A record
`sfbw   A   1.2.3.4   3600`

Install certbot, a tool that makes it easy to register new SSL certificates.
```bash
add-apt-repository ppa:certbot/certbot
apt install python-certbot-nginx
```
Configure nginx to work as a reverse proxy for both Alice's and Bob's nodes
```bash
cd /etc/nginx/sites-available
vim sfbw.bootnodes.net
```

Replace the contents of the file with this
```
server {
  listen 80;

  server_name denver.bootnodes.net;

  root /var/www/html;
  index index.html;

  location / {
    try_files $uri $uri/ =404;
  }

  location /alice {
    proxy_buffers 16 4k;
    proxy_buffer_size 2k;
    proxy_pass http://localhost:9944;
    proxy_http_version 1.1;
  }

  location /bob {
    proxy_buffers 16 4k;
    proxy_buffer_size 2k;
    proxy_pass http://localhost:9945;
    proxy_http_version 1.1;
  }

# Uncomment these lines to enable reverse proxy for http rpcs as well
#  location /alice/rpc {
#    proxy_buffers 16 4k;
#    proxy_buffer_size 2k;
#    proxy_pass http://localhost:9933;
#    proxy_http_version 1.1;
#  }

#  location /bob/rpc {
#    proxy_buffers 16 4k;
#    proxy_buffer_size 2k;
#    proxy_pass http://localhost:9934;
#    proxy_http_version 1.1;
#  }
}

```

Enable the new config by linking it from `sites-available` to `sites-enabled`.

```bash
ln -s /etc/nginx/sites-available/sfbw.bootnodes.net /etc/nginx/sites-enabled/
```

Confirm config format is ok, and if it is, reload nginx.
```bash
# Check nginx config syntax
nginx -t

# Reload the server
systemctl reload nginx
```

Use certbot to setup ssl
`certbot --nginx -d sfbw.bootnodes.net --register-unsafely-without-email`
You could also fork over your email. It only goes to EFF. I chose not to redirect http, but we should experiment with it. If it doesn't break anything, we should do it.

Confirm your site loads with ssl https://sfbw.bootnodes.net

## Build and Run Substrate Node
```bash
# First time around I did the apt/rustup installs manually
# Script also works as of 1Nov2019
curl https://getsubstrate.io -sSf | bash -s -- --fast

git clone https://github.com/substrate-developer-hub/substrate-node-template
cd substrate-node-template
cargo build --release # If cargo was _just_ installed, start a new shell so it's on your path
```

To test that our reverse proxy is working, we'll start a node at Alice's ports and confirm we can connect to it throug hApps, then do likewise for Bob's ports.

```bash
# Test Alice's ports
./target/release/node-template --dev

# Test Bob's ports
./target/release/node-template --dev --ws-port 9945 --rpc-port 9934 --port 30303
```

Confirm you can connect with hosted apps. On settings tab use `wss://sfbw.bootnodes.net/alice` and /bob.

## Create a shared chainspec
Create a basic chainspec based on local testnet
`node-template build-spec --chain local > spec.json`

Edit the name and id of the network, the root key, the prefunded accounts etc.

Before we can add bootnodes to the chainspec, we need to know their node identities. That means we need to start each node once to let it generate node keys.

Start Alice's node like
`node-template --chain=spec.json --alice`
Once the node starts, observe its node identity, then kill it with ^C.
Repeat this for any other nodes you'd like in the chainspec's bootnodes section.


Now edit the chainspec again, adding each bootnode in the format
```json
"bootNodes": [
    "/dns4/sfbw.bootnodes.net/tcp/30333/p2p/QmNdzun5tXSo7TPEntmujvU3eLEjTJKfXpJAvwp1ikpa6T",
    "/ip4/167.71.86.67/tcp/30333/p2p/QmdP4qG1ZSgzmsdFpBwuPAVWG9zjPRHV3dSkTT8v4TGP4J"
],
```

Warning: You should not delete the node's entire data directory from this point on. You may purge the chain with the `purge-chain` sub command, but if you delete the entire directory, it will delete the node key and change the node's identity.

Finally, publish the chainspec by copying it to the web directory

`cp spec.json /var/www/html/spec.json`

Comfirm you can access it over the web
`https://sfbw.bootnodes.net/spec.json`

## Startup scripts (optional)
If your nodes need many flags, it may be wise to make a startup script just so you don't mess it up live. I usually write one like this.

```bash
# Purge any old chain.
# Only wise for chains that will be restarted frequently (eg workshops)
# Long running chains should not be purged to avoid constant re-syncs
./target/release/node-template purge-chain --chain=spec.json -y

./target/release/node-template \
        --chain=spec.json \
        --alice \
        --ws-port 9994 \
        --rpc-port 9993
```

## Share the prefunded account
Remember at that our website offers users to download the pre-funded key. Add the Alice key to apps and export it to json.
[Dev phrase](https://github.com/paritytech/substrate/blob/93123cc63eac37fed7a6cc6cc58e7e43d666ee03/core/primitives/src/crypto.rs#L40)

bottom drive obey lake curtain smoke basket hold race lonely fit walk
//Alice
I use password: Alice
Upload the json key to the server

## Host a frontend
Back in /root, clone the front end template
`git clone https://github.com/substrate-developer-hub/substrate-front-end-template/`

Install yarn following https://yarnpkg.com/lang/en/docs/install/#debian-stable
```bash
curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | sudo apt-key add -
echo "deb https://dl.yarnpkg.com/debian/ stable main" | sudo tee /etc/apt/sources.list.d/yarn.list
apt update
apt install yarn
```

install dependencies `yarn`
modify production server to match your needs
`vim src/config/production.json`
and use wss://sfbw.bootnodes.net:9944

build production release with `yarn build`
then move build output directory inside of web root
`mv build /var/www/html/front-end`

This is where I'm stuck. Loading https://sfbw.bootnodes.net/front-end shows a blank page, and the console shows warnings about scripts that didn't load.

I've failed many different ways at this point, and rarely even succeeded.
