---
title: GraphQL API
---

From the [GraphQL site](https://graphql.org/learn/introduction/)

GraphQL is a query language for your API, and a server-side runtime for
executing queries using a type system you define for your data. The GraphQL
specification was open-sourced in 2015 and has since been implemented in a
variety of programming languages. GraphQL isn’t tied to any specific database or
storage engine—it is backed by your existing code and data.

`zkool_graphql` is a GraphQL server for the Zkool wallet engine. It offers a
typesafe API to wallet creation, synchronization, address generation and
payments.

It can be used to develop apps that leverage the Zcash privacy technologies.

> Unlike the classic JSON RPC, GraphQL is *object oriented*, supports *custom queries*
and *push notifications*.

## Queries

Queries are requests that return information about your wallets without making
any modification. Unless a query refers to the blockchain (which changes
externally), calling it multiple times returns the same result.

## Mutations

Mutations are request that may change the data of the wallet. For instance,
creating a new account, making a payment, or synchronization are mutations.

## Subscriptions

Subscriptions are long running queries that lets the server send notifications
to the client. The app can subscribe to events for new blocks and incoming
transactions.
