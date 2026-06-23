# Hive rApp deploy tool

Self-contained tool that builds + signs a Reality **`DeployAppTransaction`** registering a Hive
rApp on the testnet, then prints the `curl` to submit it. This is the **per-operator deploy**
(Model B): each operator registers their own rApp once, then runs Hive pointed at the same key.

It's a tiny Scala one-shot that uses the Reality SDK directly (so the transaction encoding +
signing are guaranteed correct), provided as an **unmanaged jar** — no Maven coordinates needed.

## Prerequisites
- **JDK 17** and **sbt**.
- The Reality **assembly jar** dropped at **`deploy/lib/reality-combined.jar`**
  (e.g. `reality-combined-assembly-*.jar`). It's git-ignored (~151 MB). This jar also provides
  **keytool** (`org.reality.keytool.Main`).

## Steps

```bash
cd deploy

# 1. Put the SDK jar in place
cp ~/Downloads/reality-combined-assembly-*.jar lib/reality-combined.jar

# 2. Generate your rApp signing key (PKCS12; dev creds alias=alias, password=password)
mkdir -p lib
java -cp lib/reality-combined.jar org.reality.keytool.Main generate   # confirm flags with --help
#   -> produces a key.p12 (point HIVE_KEYSTORE at it)

# 3. Configure + build the signed deploy tx (env vars; defaults match the testnet)
export HIVE_KEYSTORE=lib/key.p12
export HIVE_APP_NAME="my-shop"
export HIVE_APP_VERSION="0.3.0"
export HIVE_APP_URL="https://github.com/kalkiboru111/hive/releases/download/v0.3.0/hive-linux-x86_64"
# binaryHash defaults to the published v0.3.0 hash; or set HIVE_ARTIFACT=/path/to/hive to compute it
sbt "runMain CreateHiveDeployAppTx /tmp/hive-deploy.tx"

# 4. Submit it (the run prints this exact line)
curl -X POST http://143.110.227.9:9100/transactions -H 'Content-Type: application/json' -d @/tmp/hive-deploy.tx

# 5. Point Hive at the SAME key so its snapshots are signed by the deployed rApp's key
java -cp lib/reality-combined.jar org.reality.keytool.Main export   # -> hex private key
#   then in your bot's config.yaml:
#   network:
#     enabled: true
#     l0_url: "http://143.110.227.9:9000"
#     identity_key_hex: "<hex from keytool export>"
```

`scripts/deploy.sh` (in this dir) wraps steps 3–4.

## Environment variables

| Var | Default | Notes |
|-----|---------|-------|
| `HIVE_KEYSTORE` | `lib/key.p12` | PKCS12 keystore with your rApp key |
| `HIVE_KEY_ALIAS` / `HIVE_KEY_PASSWORD` | `alias` / `password` | keystore creds |
| `HIVE_DESTINATION` | testnet genesis `NET8Q7Y4o…` | deploy destination |
| `HIVE_APP_NAME` / `HIVE_APP_VERSION` / `HIVE_APP_DESCRIPTION` | `hive` / `0.3.0` / … | app metadata |
| `HIVE_APP_URL` | v0.3.0 release URL | where the binary is downloaded from |
| `HIVE_ARTIFACT` | — | local binary path; if set, its SHA-256 becomes `binaryHash` |
| `HIVE_BINARY_HASH` | published v0.3.0 hash | used if `HIVE_ARTIFACT` unset |
| `HIVE_TOKEN_TICKER` | `HIVE` | token symbol |
| `HIVE_TOTAL_SUPPLY` / `HIVE_TOKENS_FOR_SALE` / `HIVE_TOTAL_REWARDS` / `HIVE_STARTING_BALANCE` | 100M / 7M / 30M / 63M (×1e8) | **must satisfy** `startingBalance + totalRewards + tokensForSale == totalSupply` |
| `HIVE_TOKEN_PRICE` | `1` | raw NET per raw token |
| `HIVE_L0_TX_URL` | `http://143.110.227.9:9100/transactions` | printed submit endpoint |
