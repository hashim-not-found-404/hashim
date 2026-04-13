// the client cache should be:
// embededd and server
// if its server it should be socket
// store the data as raw data like the database schema
// optional to exist or not
// read only
// it dont transform the data

// if it optional i think i should be compatable with other trait (database) to use it for fast state full check
// it is used for read and for check
// cache and database have the same trait and the SLC and SFC and request also have the same trait
// that mean i need request and response compatable with the cache

// impl                     driving trait               dreven trait
// State                    null                        BackendRouts
// StateLessCheck           BackendRouts                BackendRouts
// StateFullCheck           BackendRouts                DBTransaction
// MyClient                 BackendRouts                null
// CockroachTxn             DBTransaction               null
// Cache                    DBTransaction , B , C       A
// FromCacheToServer        A                           null
// FromServerToCache        null                        DBTransaction
// FromClientToCache        DBTransaction               null
// FromCacheToClient        null                        DBTransaction

// impl                     driving impl                                dreven impl
// State                    null                                        StateLessCheck
// StateLessCheck           State                                       StateFullCheck , MyClient
// StateFullCheck           StateLessCheck                              CockroachTxn , Cache , FromClientToCache
// MyClient                 StateLessCheck                              null
// CockroachTxn             StateFullCheck , FromClientToCache          null
// Cache                    StateFullCheck , FromCacheToClient          FromCacheToServer
// FromCacheToServer        Cache                                       null
// FromServerToCache        null                                        CockroachTxn
// FromClientToCache        StateFullCheck                              null
// FromCacheToClient        null                                        Cache

// the reasone for the  cache to have driving trait DBTransaction is to use it in StateFullCheck
// but the dreven trait will be some thing else
// but the cache will not be the same as db for every thing because we will use to auto complete user inputs
// but the user may be use serverless choice that mean he will make his data on device that mean i dont need cache for that
// that mean the database should be the same as cache
// but if the cache is read only that mean the app is slow i think we can make it read write and sync in the background
// if it not sync it will fire error
// but to sync the data from the client to server it should first make state full check in the server
// ok i think it will be some thing like Replicache
// the communication between cache and database should follow the RBAC to prevent data leak
// ok that mean the cache will send data to ui based on RBAC and will get the data from the server based on RBAC
// ok because (the cache will send data to ui based on RBAC) that mean it will implement the DBTransaction

// the ui write to both cache and server
// the cache request from server by RBAC
// the server poke the cache
// but the cache never write to server to prevent misuse by follow RBAC
// and cache never request from server to prevent data leak

// ui -> cache
// ui -> server
// server -> cache
// cache -> ui

// ui write to cache
// ui write to server
// server write to cache
// cache write to ui

// the cache implement three trait
// one work like the data base for state full check
// and secund for get poke from server
// and third to request from server

// i think i need to seperate the state less check from the state full check from storing
// i think that because i think that will help for scalling the code base and help for adding cache
