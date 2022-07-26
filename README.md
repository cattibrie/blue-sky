# blue-sky

### Basic
Build application with the sample test file 'transactions.csv'.

```
cargo run -- transactions.csv > accounts.csv
```

The accounts.csv file has the result of this command.

Tests the application:

```
cargo test
```

### Completeness
All transaction cases are handled.

From the task description I understood that only 'Deposit' and 'Withdrawal' transactions can be claimed in 'Dispute', 'Resolve' and 'Chargeback'.

In case when 'Deposit' transaction is claimed as erroneous, the logic implemented is the one described in the task.
On every dispute related transaction ('Dispute', 'Resolve' and 'Chargeback') amount held or charged is not bigger that available.

In case when 'Withdrawal' transaction is claimed as erroneous:
- during 'Dispute' and 'Resolve' no amount is held from the client.
- during 'Chargeback' the client receives the amount that erroneously was chraged from their account.
- after 'Chargeback' the client is not locked.


Tests were added to next cases:
- check that input and output data is handled correctly (as it was asked in the task);
- separate tests cases to check every type of Transaction;
- check that dispute related transaction can happen only for successful transactions. 
The failing transaction is 'Withdrawal' that failed because there was not enough available amount. 
- check that the same dispute related transaction cannot happen twice;
- check that 'Resolve' only valid after 'Dispute' and 'Chargeback' is only valid after 'Resolve'.

Additional tests that I would add:
- several dispute transaction of the same type related to the same client but different Transactions (Deposit, Withdrawal).

### Safety and Robustness
If during the execution an error happens, the resulting info about accounts won't be printed.
Instead the proccess will fail and show the reason of failure.

It is not safe to process data if input is not valid.

It is bad user experince if a user/client gets unreadable/invalid information.

### Efficiency
Current solution won't work right if it is a service that receives concurrent TCP streams. 
In this case there should be a lock assosiated with every client.
All the changes to a client should happen at the same time for every transaction before next transaction related to the same client may happen.
What is chronological order between transaction from different streams?

