import cats.effect.{IO, IOApp, ExitCode}

import io.circe.syntax.*
import io.github.iltotore.iron.*
import io.github.iltotore.iron.constraint.numeric.{Positive, Positive0}

import java.io.FileInputStream
import java.nio.file.{Files => JFiles, Paths => JPaths}
import java.security.{KeyPair, KeyStore, MessageDigest, PrivateKey}
import scala.collection.immutable.SortedMap

import org.reality.ext.crypto.*
import org.reality.schema.address.Address
import org.reality.schema.balance.{Amount, Balance}
import org.reality.schema.netAddress.NETAddress
import org.reality.schema.transaction.{
  DeployAppTransaction,
  Transaction,
  TransactionAmount,
  TransactionFee,
  TransactionReference,
  TransactionSalt
}
import org.reality.security.SecureRandom
import org.reality.security.SecurityProvider
import org.reality.security.signature.Signed

/** Builds + signs a Reality `DeployAppTransaction` registering a Hive rApp, and
 *  writes the signed tx JSON ready to POST to the testnet.
 *
 *  This is the per-operator deploy tool (Model B). All inputs come from
 *  environment variables (defaults match the verified testnet values). Run with:
 *    sbt "runMain CreateHiveDeployAppTx [out-path]"   (default /tmp/hive-deploy.tx)
 *
 *  Reality SDK comes from deploy/lib/reality-combined.jar (see README.md).
 *
 *  Token amounts are RAW units (1 token = 1e8 raw). The network enforces:
 *    startingBalances + totalRewards + tokensForSale == totalSupply
 */
object CreateHiveDeployAppTx extends IOApp {

  private def env(k: String, d: String): String = sys.env.getOrElse(k, d)
  private def envLong(k: String, d: Long): Long = sys.env.get(k).map(_.trim.toLong).getOrElse(d)

  // ── Keystore (your rApp signing key — generate with Reality keytool) ─────────
  private val KeystorePath = env("HIVE_KEYSTORE", "lib/key.p12")
  private val KeyAlias      = env("HIVE_KEY_ALIAS", "alias")
  private val KeyPassword   = env("HIVE_KEY_PASSWORD", "password")

  // ── App identity ─────────────────────────────────────────────────────────────
  private val DestinationAddr = env("HIVE_DESTINATION", "NET8Q7Y4oZxXwZZz5Qze3y9tHDs9PP8TprEZJ8yf") // testnet genesis
  private val AppName         = env("HIVE_APP_NAME", "hive")
  private val AppVersion      = env("HIVE_APP_VERSION", "0.3.0")
  private val AppDescription  = env("HIVE_APP_DESCRIPTION", "Hive WhatsApp commerce bot")
  private val AppDownloadURL  = env(
    "HIVE_APP_URL",
    "https://github.com/kalkiboru111/hive/releases/download/v0.3.0/hive-linux-x86_64"
  )
  // Either point HIVE_ARTIFACT at a local binary (its SHA-256 is computed) or set
  // HIVE_BINARY_HASH directly (defaults to the published v0.3.0 hash).
  private val ArtifactPath = sys.env.get("HIVE_ARTIFACT")
  private val BinaryHashEnv = env(
    "HIVE_BINARY_HASH",
    "8a40be483ed55de5376de16e75be0d26a2c2c83283c5ef7771f032e0871beeca"
  )

  // ── Token economics (raw ×1e8). Partition rule enforced below. ───────────────
  private val TotalSupply     = envLong("HIVE_TOTAL_SUPPLY", 10_000_000_000_000_000L) // 100M
  private val TokenPrice      = envLong("HIVE_TOKEN_PRICE", 1L)
  private val TokensForSale   = envLong("HIVE_TOKENS_FOR_SALE", 700_000_000_000_000L) //  7M
  private val TotalRewards    = envLong("HIVE_TOTAL_REWARDS", 3_000_000_000_000_000L) // 30M
  private val StartingBalance = envLong("HIVE_STARTING_BALANCE", 6_300_000_000_000_000L) // 63M to deployer
  private val TimeLimitOrdinalDiff = envLong("HIVE_TIME_LIMIT_ORDINAL_DIFF", 0L)
  private val TokenTicker     = env("HIVE_TOKEN_TICKER", "HIVE")

  private val TxEndpoint = env("HIVE_L0_TX_URL", "http://143.110.227.9:9100/transactions")

  override def run(args: List[String]): IO[ExitCode] = {
    val outPath = args.headOption.getOrElse("/tmp/hive-deploy.tx")

    SecurityProvider.forAsync[IO].use { implicit sp =>
      for {
        keyPair <- IO.delay(loadKeyPair(KeystorePath, KeyAlias, KeyPassword))

        sourceAddr <- IO.fromEither(
          NETAddress.either(Address.buildFrom(keyPair.getPublic).value.value)
            .left.map(new RuntimeException(_))
        ).map(Address.apply)
        destinationAddr <- IO.fromEither(
          NETAddress.either(DestinationAddr).left.map(new RuntimeException(_))
        ).map(Address.apply)

        startingBalances <- IO.fromEither {
          Balance.either(StartingBalance)
            .map(b => SortedMap(sourceAddr -> b))
            .left.map(new RuntimeException(_))
        }

        partitionSum = StartingBalance + TotalRewards + TokensForSale
        _ <- IO.println(s"partition sum: $partitionSum  totalSupply: $TotalSupply")
        _ <- IO.raiseWhen(partitionSum != TotalSupply)(
          new RuntimeException(
            s"startingBalances($StartingBalance) + totalRewards($TotalRewards) + tokensForSale($TokensForSale) " +
              s"= $partitionSum != totalSupply($TotalSupply) — refusing to sign"
          )
        )

        binaryHash = ArtifactPath match {
          case Some(p) if JFiles.exists(JPaths.get(p)) => sha256Hex(JFiles.readAllBytes(JPaths.get(p)))
          case _                                       => BinaryHashEnv
        }
        _ <- IO.println(s"binaryHash: $binaryHash")

        salt <- SecureRandom.get[IO].map(_.nextLong()).map(TransactionSalt.apply)

        tx = DeployAppTransaction(
          source = sourceAddr,
          destination = destinationAddr,
          binaryHash = binaryHash,
          appName = AppName,
          appVersion = AppVersion,
          appDescription = AppDescription,
          appDownloadURL = AppDownloadURL,
          fee = TransactionFee(Amount.empty),
          amount = TransactionAmount(Amount.empty),
          parent = TransactionReference.empty,
          salt = salt,
          tokenTicker = TokenTicker,
          totalSupply = TotalSupply.refineUnsafe[Positive],
          tokenPrice = TokenPrice.refineUnsafe[Positive],
          tokensForSale = TokensForSale.refineUnsafe[Positive],
          startingBalances = startingBalances,
          totalRewards = TotalRewards.refineUnsafe[Positive0],
          timeLimitOrdinalDiff = TimeLimitOrdinalDiff.refineUnsafe[Positive0]
        )

        signed <- Signed.forAsyncJson[IO, Transaction](tx, keyPair)
        _ <- IO.delay(
          JFiles.write(JPaths.get(outPath), (signed: Signed[Transaction]).asJson.noSpaces.getBytes("UTF-8"))
        )
        _ <- IO.println("")
        _ <- IO.println(s"rApp (source) address = ${sourceAddr.value.value}")
        _ <- IO.println(s"app = $AppName v$AppVersion   ticker = $TokenTicker")
        _ <- IO.println(s"wrote signed deploy tx -> $outPath")
        _ <- IO.println("")
        _ <- IO.println("Submit with:")
        _ <- IO.println(s"  curl -X POST $TxEndpoint -H 'Content-Type: application/json' -d @$outPath")
      } yield ExitCode.Success
    }
  }

  private def sha256Hex(bytes: Array[Byte]): String = {
    val md = MessageDigest.getInstance("SHA-256")
    md.update(bytes)
    md.digest().map("%02x".format(_)).mkString
  }

  private def loadKeyPair(path: String, alias: String, password: String): KeyPair = {
    val ks = KeyStore.getInstance("PKCS12")
    val fis = new FileInputStream(path)
    try ks.load(fis, password.toCharArray) finally fis.close()
    val privateKey = ks.getKey(alias, password.toCharArray).asInstanceOf[PrivateKey]
    val cert = ks.getCertificate(alias)
    new KeyPair(cert.getPublicKey, privateKey)
  }
}
