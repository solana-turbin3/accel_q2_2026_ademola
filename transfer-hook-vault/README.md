VAULT IMPLEMENTING TOKEN EXTENSIONS' TRANSFER HOOK TO LIMIT VAULT INTERACTIONS TO ONLY WHITELISTED ADDRESSES/USERS

APPROACH
[] initiate vault:
  this involves creating a vault config with admin, and mint with transfer hook extension

[] whitelist addresses/users:
  this involves adding addresses/users to the whitelist
[] Deposit tokens:
  this allows whitelisted users to deposit tokens into the vault
[] Withdraw tokens:
  this allows whitelisted users to withdraw tokens from the vault
[] transfer tokens:
  this allows whitelisted users to transfer tokens from the vault
[] remove addresses/users from whitelist:
  this allows admin to remove addresses/users from the whitelist
