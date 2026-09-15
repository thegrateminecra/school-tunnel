# School Tunnel

Routes your school laptop internet through Cloudflare so you can access blocked sites.

## Quick Setup (30 seconds)

1. Download `tunnel-client.exe` and `start.bat` from this repo
2. Put them in the same folder on your laptop
3. Double-click `start.bat`
4. Set your browser proxy (see below)
5. Browse normally

## Browser Proxy Setup (Firefox)

1. Open Firefox → Settings (hamburger menu → Settings)
2. Scroll down to **Network Settings** → click **Settings...**
3. Select **Manual proxy configuration**
4. Fill in:
   - SOCKS Host: `127.0.0.1`
   - SOCKS Port: `1080`
5. **Check the box**: Proxy DNS when using SOCKS v5
6. Click **OK**

## What This Does

```
Your Laptop  →  Cloudflare Worker  →  Website
     |                |                   |
  (SOCKS5)      (WebSocket+TLS)       (Normal)
```

Your browser traffic goes through a Cloudflare Worker (a serverless function on Cloudflare's network). ContentKeeper sees a connection to `*.workers.dev` which is a legitimate Cloudflare domain.

## Commands

Run manually if you don't want to use the bat file:

```
tunnel-client.exe -s tunnel-proxy.tgm-pub.workers.dev -t F3QHNvt6fIDwg9bJA4pOSqjx
```

## Troubleshooting

**"proxy server refusing connections"**
- Make sure `tunnel-client.exe` is running (the black terminal window should be open)
- Make sure Firefox proxy is set to SOCKS 127.0.0.1:1080

**Pages won't load**
- Check that the terminal window says "Tunnel active"
- Try restarting the bat file

**Certificate warning**
- This is normal — ContentKeeper is doing its SSL inspection thing. The tunnel still works through it.
