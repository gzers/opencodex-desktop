import { describe, expect, it } from 'vitest'
import { displayPath, localFileName, localPathExample } from '@/lib/paths'

describe('shared local path presentation', () => {
  it.each([
    ['\\\\?\\D:\\My App\\data/backups', 'D:\\My App\\data\\backups'],
    ['D:/应用目录/My App/file.json', 'D:\\应用目录\\My App\\file.json'],
    ['\\\\?\\UNC\\server\\share\\My App/backups', '\\\\server\\share\\My App\\backups'],
    ['\\\\server\\share\\My App/backups', '\\\\server\\share\\My App\\backups'],
    ['D:\\', 'D:\\'],
    ['\\\\?\\D:\\', 'D:\\'],
    ['\\\\?\\UNC\\server\\share', '\\\\server\\share'],
    ['D:\\parent & more\\file.json', 'D:\\parent & more\\file.json'],
  ])('displays an ordinary local identity %s', (value, expected) => {
    expect(displayPath(value, 'Windows')).toBe(expected)
    expect(displayPath(expected)).toBe(expected)
  })
  it.each([
    '\\\\?\\GLOBALROOT\\Device\\HarddiskVolume1',
    '\\\\.\\pipe\\opencodex',
    '\\\\?\\Volume{00000000-0000-0000-0000-000000000000}\\data',
    '\\\\?\\D:\\NUL\\file',
    '\\\\?\\D:\\COM¹\\file',
    '\\\\?\\D:\\ending.\\file',
    '\\\\?\\D:\\ending \\file',
    '\\\\?\\D:\\base\\..\\file',
    '\\\\?\\D:\\base\\.\\file',
    '\\\\?\\D:\\file:stream',
    ' D:\\My App\\file',
    'D:\\My App\\file ',
    'D:relative',
    '\\root-relative',
    'relative\\file',
    'https://server/path/to/file',
    'file:///D:/My%20App/file',
    '/Users/My App/file',
    '/tmp/D:/file',
    'npm error: D:\\My App\\file',
  ])('preserves namespaces, special identities and non-local values %s', value => {
    expect(displayPath(value)).toBe(value)
  })
  it('retains extended long paths and POSIX backslashes', () => {
    const long = '\\\\?\\D:\\' + 'long directory\\'.repeat(25) + 'file'
    expect(displayPath(long)).toBe(long)
    expect(displayPath('D:\\file', 'macOS')).toBe('D:\\file')
    expect(displayPath('/Users/back\\slash', 'macOS')).toBe('/Users/back\\slash')
    expect(displayPath('//server/share/POSIX')).toBe('//server/share/POSIX')
    expect(displayPath('//server/share/Windows', 'Windows')).toBe('\\\\server\\share\\Windows')
    expect(displayPath(null)).toBe('')
  })
  it('extracts selected filenames and uses native examples without changing URLs', () => {
    expect(localFileName('D:\\My App\\opencodex.tgz')).toBe('opencodex.tgz')
    expect(localFileName('/Users/My App/opencodex.tgz')).toBe('opencodex.tgz')
    expect(localPathExample('Data', 'Windows')).toBe('D:\\Data')
    expect(localPathExample('Data', 'macOS')).toBe('/absolute/path/to/Data')
  })
})
