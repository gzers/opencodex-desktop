// Only production-owned channels are admitted; no endpoint override exists.
export function otaChannel(value='stable'){
 if(!['stable','beta'].includes(value))throw Error('OTA acceptance channel must be stable or beta')
 return value
}
