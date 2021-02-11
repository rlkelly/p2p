# p2p

- limit bytes taken in buffer (this is breaking)
- validate music files
- add a public key and signature to all requests as a reference.
- Rewrite storage to manage users based on public key
- write data structure for storing and assembling chunks
- add time without a ping to users to control dropping them?
   - drop after certain time
   - validate with some frequency
