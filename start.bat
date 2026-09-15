@echo off
title School Tunnel
echo ========================================
echo   Starting proxy tunnel...
echo ========================================
echo.
echo   Server: tunnel-proxy.tgm-pub.workers.dev
echo   Token:  F3QHNvt6fIDwg9bJA4pOSqjx
echo.
echo   Browser proxy settings:
echo     SOCKS Host: 127.0.0.1
echo     SOCKS Port: 1080
echo     [x] Proxy DNS when using SOCKS v5
echo.
echo ========================================
echo.
tunnel-client.exe -s tunnel-proxy.tgm-pub.workers.dev -t F3QHNvt6fIDwg9bJA4pOSqjx
pause
