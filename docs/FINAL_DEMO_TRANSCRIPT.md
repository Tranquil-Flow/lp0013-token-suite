# LP-0013 final demo read-aloud transcript

Read this while running `bash scripts/record-final-video.sh` on the M4 environment with `wallet` available.

## Opening

This is the final LP-0013 token authorities resubmission demo. I am showing the corrected public-testnet evidence for mint authority lifecycle support on LEZ tokens.

The important correction is that holding account creation and minting are separate operations. That means `mint_to` mutates an existing holding instead of accidentally depending on initialization side effects.

## Canonical evidence

The canonical Program ID and Image ID are `32335764e583cd45684e0100ca63a3564a02274daa6ea6a5f758fad671b0a9ce`. The mint PDA is `HtCYkKN5K3dUVnPhJ4tCNpvDrnEcLZKgh8i4PkUjigfu`.

I am running the repository validator first, then the read-only public-testnet verifier. This verifier queries the live public sequencer and does not submit new transactions.

## Public-testnet lifecycle

The verifier shows seven transaction outcomes. The program was deployed. A mint was created. A holding account was created. Then `mint_to(60)` succeeded, `mint_to(40)` succeeded, and the supply accumulated to one hundred.

Next, `set_mint_authority(None)` revoked the mint authority. Finally, the post-revoke `mint_to` was rejected and did not land on chain.

## Closing

The final mint state is authority `None`, supply `100`, and decimals `6`. This proves variable supply before revocation and permanent authority revocation afterwards.

This video supersedes the older pre-fix evidence. The resubmission should cite this corrected public-testnet run only.
