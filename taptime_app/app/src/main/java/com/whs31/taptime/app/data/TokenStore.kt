package com.whs31.taptime.app.data

interface TokenStore {
    fun read(): String?
    fun write(token: String)
    fun clear()
}
