import {test} from 'node:test'
import assert from 'node:assert/strict'
import {otaChannel} from './windows-ota-channel.mjs'
test('production stable remains the default; the existing beta channel is explicit',()=>{
 assert.equal(otaChannel(),'stable');assert.equal(otaChannel('stable'),'stable');assert.equal(otaChannel('beta'),'beta')
})
test('arbitrary URLs, fallback channels and malformed values cannot replace the real endpoint',()=>{
 for(const value of ['https://example.test/latest.json','http://127.0.0.1/','nightly','BETA','beta ','',null,{}])assert.throws(()=>otaChannel(value))
})
