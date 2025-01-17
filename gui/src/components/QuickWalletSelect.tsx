import { MenuItem, Select, SelectChangeEvent } from "@mui/material";

import { useWallets } from "../hooks/useWallets";

export function QuickWalletSelect() {
  const { wallets, currentWallet, setCurrentWallet } = useWallets();

  const handleChange = (event: SelectChangeEvent<string>) => {
    setCurrentWallet(event.target.value);
  };

  if (!wallets || !currentWallet) return <>Loading</>;

  return (
    <Select size="small" value={currentWallet.name} onChange={handleChange}>
      {wallets.map(({ name }) => (
        <MenuItem value={name} key={name}>
          {name}
        </MenuItem>
      ))}
    </Select>
  );
}
