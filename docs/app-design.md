this is a cross border payments app.

users create fx money movements on behalf of other businesses. so this is b2b2b.

# Abbreviations

- LJE: ledger journal entry

# Entities

## ledger entries
- many to one relation to ledger_journal_entries

## ledger journal entries
- the parent entity for ledger entries, aggregates entries into groups of zero-sum entries. (sum of credit === sum of debits)
- ledger journal entries have fk's to transfers and parties.

## transfers
- the core primitive of this fintech app.
- we will start with just remittance (cross border transfers)

## parties
- participants in a transfer
- for remittance, the originator and counterparty are parties.
- originator is the party of the transfer that initiates it.
- the originator can be either the sender OR recipient. the counterparty is simply the other party.
- party table will have
  - tax_id text column
  - legal_name text column
  - address 

## transfers
- transfers have a one to many with LJE (ledger journal entries)
- properties:
- direction (payout or payin)
- start_currency
- end_currency
- start_amount
- end_amount


# DB Schemas

## transfers
- pk
- created_by (fk to user)
- direction (payout or payin)
- start_currency
- end_currency
- start_amount
- end_amount

## ledger_journal_entry db schema
- pk
- status
- type (deposit, disbursement, int_fx, int
- fk to transfers

## ledger_entries
- pk
- fk to ledger_journal_entry
- fk to account_id
- amount
- direction
- currency_code

## ledger_accounts
- pk
- type (nostro, user, system)
- name
- is_negative_balance_allowed

## ledger_account_balances
- fk to ledger_accounts
- fk to currencies
- pending_balance numeric column
- available_balance numeric column
- posted_balance numeric column

## currencies
- code (pk, 2 letter iso code)

## parties
- pk
- tax_id
- legal_name
- address
