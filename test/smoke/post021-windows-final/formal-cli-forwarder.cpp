#define UNICODE
#include <windows.h>
#include <process.h>
#include <string>
#include <vector>
// Native launcher only: forward real status/version/cache commands to the
// exact published CLI with its Bun runtime. Never synthesize status/restart.
int wmain(int argc,wchar_t**argv){
 wchar_t path[32768];DWORD n=GetModuleFileNameW(nullptr,path,32768);if(!n||n>=32768)return 80;
 std::wstring dir(path,n);dir=dir.substr(0,dir.find_last_of(L"\\/"));
 std::wstring bun=dir+L"\\bun.exe",script=dir+L"\\official-cli-path.txt";
 HANDLE file=CreateFileW(script.c_str(),GENERIC_READ,FILE_SHARE_READ,nullptr,OPEN_EXISTING,0,nullptr);if(file==INVALID_HANDLE_VALUE)return 81;
 wchar_t buf[32768];DWORD bytes=0;BOOL ok=ReadFile(file,buf,sizeof(buf)-sizeof(wchar_t),&bytes,nullptr);CloseHandle(file);if(!ok||bytes%2)return 82;
 std::wstring cli(buf,bytes/2);if(cli.empty()||cli.find(L'\0')!=std::wstring::npos)return 83;
 std::vector<const wchar_t*>args={bun.c_str(),cli.c_str()};for(int i=1;i<argc;++i)args.push_back(argv[i]);args.push_back(nullptr);
 intptr_t result=_wspawnv(_P_WAIT,bun.c_str(),args.data());return result==-1?84:(int)result;
}
