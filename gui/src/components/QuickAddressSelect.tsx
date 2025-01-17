import { MenuItem, Select, SelectChangeEvent } from "@mui/material";
import { map } from "lodash-es";

import { useInvoke } from "../hooks/tauri";
import { useWallets } from "../hooks/useWallets";
import { Address } from "../types";
import { AddressView } from "./AddressView";

export function QuickAddressSelect() {
  const { currentWallet, setCurrentAddress } = useWallets();
  const { data: addresses } = useInvoke<[string | undefined, Address][]>(
    "wallets_get_wallet_addresses",
    { name: currentWallet?.name }
  );

  const handleChange = (event: SelectChangeEvent<string | undefined>) => {
    if (!event.target.value) return;
    setCurrentAddress(event.target.value);
  };

  const renderValue = (v: string) => {
    const address = addresses?.find(([key]) => key === v)?.[1];
    return address && <AddressView contextMenu={false} address={address} />;
  };

  if (!addresses || !currentWallet) return <>Loading</>;

  return (
    <Select
      size="small"
      renderValue={renderValue}
      value={currentWallet.currentPath || addresses[0][0]}
      onChange={handleChange}
    >
      {map(addresses, ([key, address]) => (
        <MenuItem value={key} key={key}>
          <AddressView contextMenu={false} address={address} />
        </MenuItem>
      ))}
    </Select>
  );
}
