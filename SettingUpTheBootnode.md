
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
Domain name for the cert - I register bootnodes .net
Reverse proxy - nginx
Academy PoW node - copied in binary

The ssl cert and reverse proxy are all so that students can use UIs like Apps from the hosted website without warnings or errors.
If this is not important to you, you can save some trouble by skipping the SSL stuff.

## Base System Setup

Create a new VPS starting with Ubuntu 22.04 with SSH permissions.
I use a digital ocean droplet for these purposes.
The hardware does not need to be too beefy.

* 8GB memory
* 4 AMD CPUs
* 160 GB NVMe
* 5TB transfer

Many of the following commands need to be run as root, so it is convenient to start a persistent sudo session with `sudo -i`.

Before installing anything specific, I do general OS updates.

```bash
apt update
apt upgrade
```

## Nginx Webserver and Reverse Proxy

We will use Nginx as a reverse proxy and to host a simple web page that points users to relevant sites like the UI and the github repo.

We use `apt` start by installing it.

```bash
apt install nginx
```

Confirm the server is working by going directly to your IP in the browser.
You should see the nginx test page.
The file you just viewed lives at `/var/www/html/index.nginx-debian.html`.
This file is only used when a regular `index.html` file is not present.
Create your own index file at `/var/www/html/index.html` with content like the following.

```html
<!DOCTYPE html>
<html>
	<head>
		<title>Academy PoW Blockchain Network</title>
		<style>
			body {
				width: 45em;
				margin: 0 auto;
			}
		</style>
	</head>
	<body>
		<h1>Academy PoW Blockchain Network</h1>
		<ul>
			<li>Clone the <a href="https://github.com/Polkadot-Blockchain-Academy/Academy-PoW">Academy PoW</a> project on github.</li>
			<li>Connect a <a href="https://polkadot.js.org/apps?rpc=wss://academy.bootnodes.net/websocket">Wallet to the bootnode</a>.</li>
			<li>Connect a <a href="https://polkadot.js.org/apps?rpc=ws://127.0.0.1:9944">Wallet to your local node</a>.</li>
			<li>View the <a href="https://telemetry.polkadot.io/#list/TODO_GENESIS_BLOCK_HASH">Node Telemetry</a>.</li>
		</ul>
	</body>
</html>
```

We'll make those links work shortly.
For now just confirm the new webpage loads.

## Domain Name

In order for a certificate authority to sign your SSL certificate, you will need a properly registered domain name.
Choose a registrar and pay the fee.
Specify the name servers for your VPS; digital ocean in my case.
This process varies a lot by registrar.
When your domain loads the webpage we just created, you may proceed to setup SSL.

In some cases you might want to use a subdomain.
For example, I usually use academy.bootnodes.net
This just takes an additional A record at your host.
`academy   A   1.2.3.4   3600`

## Adding SSL

Install certbot, a tool that makes it easy to register new SSL certificates.
```bash
snap install --classic certbot
```
Configure nginx to work as a reverse proxy for the bootnode that we will start shortly.
```bash
cd /etc/nginx/sites-available
vim academy.bootnodes.net
```

Replace the contents of the file with this
```
server {
  listen 80;

  server_name academy.bootnodes.net;

  root /var/www/html;
  index index.html;

  location / {
    try_files $uri $uri/ =404;
  }

  location /websocket {
    proxy_buffers 16 4k;
    proxy_buffer_size 2k;
    proxy_pass http://localhost:9944;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
    proxy_read_timeout 86400;
  }
}

```

Enable the new config by linking it from `sites-available` to `sites-enabled`.

```bash
ln -s /etc/nginx/sites-available/academy.bootnodes.net /etc/nginx/sites-enabled/
```

Confirm config format is ok, and if it is, reload nginx.
```bash
# Check nginx config syntax
nginx -t

# Reload or restart the server (or both for good measure)
systemctl reload nginx
systemctl restart nginx
```

Use certbot to setup ssl.
Notice that this command modifies the nginx config you just made in the previous step.
You can look back at it if you are curious.
```bash
certbot --nginx -d academy.bootnodes.net --register-unsafely-without-email
```

Confirm your site loads with ssl by visiting https://academy.bootnodes.net

## Academy PoW Node

At this point the server is configured and you just need to start your blockchain node.
You have a few options for getting a node onto your server including:
* Compile it on the server
* Copy a compiled binary to the server
* Pull a docker image

I happen to have the same x86-64 architecture as the server, so I usually compile locally and copy the binary up with `scp`.

Once you have the node up, run whatever command you need to.
For a simple dev chain you could do `academy-pow --dev` but for a live workshop, you probably want a dedicated spec baked into the node.

You should now have a working node!
Confirm you can:
* connect the hosted Apps UI by visiting https://polkadot.js.org/apps?rpc=wss://academy.bootnodes.net/websocket
* peer with the bootnode from a local node by using the `--bootnodes` flag.

## Telemetry Link

The last thing to do is make the telemetry link work.
The telemetry app distinguishes chains based on genesis block hash.
The easiest thing is to go to https://telemetry.polkadot.io/ and search for your chain name.
Once you find it, copy the link back into your `/var/www/html/index.html` file.