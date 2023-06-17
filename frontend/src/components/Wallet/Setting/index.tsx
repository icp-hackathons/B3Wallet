import { Stack, Text } from "@chakra-ui/react"
import { WalletSettings } from "declarations/b3_wallet/b3_wallet.did"
import { B3System, B3Wallet } from "service/actor"
import Controllers from "./Controllers"
import Cycles from "./Cycles"
import ResetAccount from "./ResetAccount"
import RestoreAccount from "./RestoreAccount"
import Signers, { SignerMap } from "./Signers"
import Status from "./Status"
import Wasm from "./Wasm"

interface SettingsProps {
  refreshWallet: () => void
  fetchAccounts: () => void
  setLoading: (loading: boolean) => void
  signers: SignerMap
  settings: WalletSettings
  actor: B3Wallet
  systemActor: B3System
}

const Settings: React.FC<SettingsProps> = ({
  actor,
  settings,
  signers,
  setLoading,
  systemActor,
  fetchAccounts,
  refreshWallet
}) => {
  return (
    <Stack spacing={4}>
      <Text fontSize="xl" fontWeight="bold">
        Settings
      </Text>
      <Cycles actor={actor} />
      <Signers actor={actor} refetch={refreshWallet} signers={signers} />
      <Stack position="relative" spacing={4}>
        <Controllers
          actor={actor}
          controllers={settings?.controllers}
          refetch={refreshWallet}
        />
      </Stack>
      <Wasm
        refreshWallet={refreshWallet}
        systemActor={systemActor}
        actor={actor}
        setLoading={setLoading}
      />
      <RestoreAccount actor={actor} fetchAccounts={fetchAccounts} />
      <Status actor={actor} />
      <ResetAccount actor={actor} fetchAccounts={fetchAccounts} />
    </Stack>
  )
}

export default Settings
