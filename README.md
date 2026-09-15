# School Tunnel

Routes your school laptop internet through Cloudflare so you can access blocked sites.

## Quick Setup (10 seconds)

1. Download `tunnel-client.exe` from this repo
2. Double-click it
3. Set Firefox proxy (one-time, see below)
4. Browse

That's it. Server address and token are built in.

## Browser Proxy Setup (Firefox, one-time)

1. Open Firefox → Settings → **Network Settings** → **Settings...**
2. Select **Manual proxy configuration**
3. SOCKS Host: `127.0.0.1` / SOCKS Port: `1080`
4. Check: **Proxy DNS when using SOCKS v5**
5. OK

## What This Does

```
Your Laptop  →  Cloudflare Worker  →  Website
     |                |                   |
  (SOCKS5)      (WebSocket+TLS)       (Normal)
```

Traffic goes through a Cloudflare Worker. ContentKeeper sees a connection to `*.workers.dev` (legitimate Cloudflare domain).

## Notes

- First run binds to your machine (HWID). Won't work on any other PC.
- Just double-click the exe to start. Ctrl+C to stop.
- The exe runs a local SOCKS5 proxy on port 1080 that Firefox connects through.

## Troubleshooting

**"proxy server refusing connections"** — Make sure the exe is running (black terminal window open) and Firefox proxy is set to SOCKS 127.0.0.1:1080

**Certificate warning** — Normal, ContentKeeper does SSL inspection. Tunnel still works through it.
