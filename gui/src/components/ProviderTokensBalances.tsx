import { useRegisterActions } from "kbar";
import { ReactNode, createContext } from "react";

import { useAccount } from "../hooks";
import { useInvoke } from "../hooks/tauri";
import { useNetworks } from "../hooks/useNetworks";
import { useRefreshTransactions } from "../hooks/useRefreshTransactions";
import { Address, TokenBalance } from "../types";

interface Value {
  balances: TokenBalance[];
  refetchBalances: () => Promise<void>;
}

export const TokensBalancesContext = createContext<Value>({} as Value);

const actionId = "token-balances";

export function ProviderTokensBalances({ children }: { children: ReactNode }) {
  const address = useAccount();
  const { currentNetwork } = useNetworks();
  const chainId = currentNetwork?.chain_id;

  const { data: balances, mutate: mutateBalances } = useInvoke<
    [Address, string][]
  >("db_get_erc20_balances", { address });

  const { mutate: refetchTokensBalances } = useInvoke(
    "alchemy_fetch_balances",
    { chainId, address },
    { refreshInterval: 10 * 1000 * 60 }
  );

  const value = {
    balances: (balances || [])?.map(([a, c]) => [a, BigInt(c)]),
    refetchBalances: async () => {
      refetchTokensBalances();
    },
  } as Value;

  useRefreshTransactions(mutateBalances);

  useRegisterActions(
    [
      {
        id: actionId,
        name: "Refresh tokens balances",
        perform: () => value.refetchBalances(),
      },
    ],
    [balances, value.refetchBalances]
  );

  return (
    <TokensBalancesContext.Provider value={value}>
      {children}
    </TokensBalancesContext.Provider>
  );
}
