# Deploy your Hive rApp to the Reality testnet

Each Hive bot instance is **its own rApp** and must be registered on-chain before the network
will accept its state-channel snapshots. Hive is the *app being posted* — it does **not** deploy
itself. Deployment is done with **Reality's own tooling** (`keytool` + the `wallet` CLI from the
[reality](https://github.com/reality-foundation/reality) repo), then you point Hive at the same
key it was deployed with.

Testnet global L0 genesis node: **`http://143.110.227.9:9000`**

> Exact CLI flag spellings come from the reality repo
> (`modules/wallet/.../cli/method.scala`, `modules/wallet/.../cli/env.scala`). Run each command
> with `--help` against your build to confirm, and check values like `--destination` with your
> testnet operator. Deploy is **free on the testnet** (`fee`/`amount` default to `0`).

## Prerequisites

- **JDK 17** (the reality tooling is JVM/Scala).
- The **reality repo** built (for `keytool` + `wallet`), e.g. `sbt wallet/assembly keytool/assembly`,
  or jars your operator provides.
- Your **Hive binary** published at a public URL (the GitHub release asset) — the deploy records
  its SHA-256 (`binaryHash`) and `appDownloadURL`.

## Steps

### 1. Generate your rApp key

```bash
java -jar keytool.jar generate            # confirm flags with: java -jar keytool.jar --help
# produces a P12 keystore, e.g. key.p12 (note its alias + password)
```

This keypair *is* your rApp identity; its address is your rApp address.

### 2. Create the signed deploy transaction

`create-deploy-app-transaction` signs the tx with your keystore and writes it to `--nextTxPath`
(it computes `binaryHash` from `--appDataPath` itself):

```bash
java -jar wallet.jar create-deploy-app-transaction \
  --keystore key.p12 --alias <alias> --password <password> \
  --destination <DESTINATION_NET_ADDRESS> \
  --appDataPath ./hive \
  --appName "my-hive-bot" \
  --appVersion "0.2.0" \
  --appDescription "Hive WhatsApp commerce bot" \
  --appDownloadURL "https://github.com/kalkiboru111/hive/releases/download/v0.2.0/hive-linux-x86_64" \
  --tokenTicker "MYBOT" \
  --totalSupply 1000000 \
  --tokenPrice 1 \
  --tokensForSale 0 \
  --nextTxPath ./deploy-tx.json
# fee/amount/totalRewards/timeLimitOrdinalDiff default to 0
```

### 3. Post it to the testnet L0

```bash
curl -X POST http://143.110.227.9:9000/transactions \
  -H 'Content-Type: application/json' \
  --data @./deploy-tx.json
```

### 4. Point Hive at the same key

Export your key and configure Hive so its snapshot signatures match the deployed rApp address:

```bash
java -jar keytool.jar export --keystore key.p12 --alias <alias> --password <password>
# copy the hex private key
```

In your bot's `config.yaml`:

```yaml
network:
  enabled: true
  l0_url: "http://143.110.227.9:9000"
  identity_key_hex: "<hex private key from keytool export>"
```

### 5. Verify

- Query the L0 for your registered app (the app-data endpoint), or
- `./hive run my-bot/` and confirm the logs show snapshots **accepted** and the global ordinal
  advancing.

You can also smoke-test the snapshot path without WhatsApp:

```bash
REALITY_URL=http://143.110.227.9:9000 cargo run --example test_reality
```

## Helper

`scripts/deploy.sh` wraps steps 2-3; set the variables at the top (or via env) and run it.

## Securing the admin dashboard

The dashboard exposes orders, customer data, and voucher/payment actions. By default it binds
to **`127.0.0.1`** (loopback only), so it is not reachable from other machines. If you need
remote access:

```yaml
dashboard:
  port: 8080
  bind_host: "0.0.0.0"        # expose on the network
  auth_token: "<long-random-secret>"   # require HTTP Basic auth (any user; password = token)
```

With `auth_token` set, all admin routes require Basic auth; the M-Pesa webhook callbacks stay
public so Safaricom can reach them. For production, terminate TLS at a reverse proxy in front of
the dashboard (Basic auth over plain HTTP sends credentials base64-encoded, not encrypted).
hive logs a warning if you bind to a non-loopback address without an `auth_token`.

## Reference (reality repo)

- `modules/wallet/.../transaction/package.scala` — `createDeployAppTransaction` (build + sign).
- `modules/tools/.../Main.scala` — `postTransaction` → `POST {baseUrl}/transactions`.
- `modules/shared/.../schema/transaction.scala` — `DeployAppTransaction` schema + signing encoding.
- `modules/shared/.../utils.scala` — `binaryHash` = SHA-256 hex.
