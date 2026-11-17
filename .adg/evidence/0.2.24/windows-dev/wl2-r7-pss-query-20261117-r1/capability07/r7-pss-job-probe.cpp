// Initial TEAM19 PSS capability probe, self-owned broker/anonymous Job only.
// No CLI accepts a target PID. No VA clone, names, memory/thread-context dump,
// external cleanup, SAME_ACCESS or CLOSE_SOURCE is used by the observer.
#define WIN32_LEAN_AND_MEAN
#define _WIN32_WINNT 0x0A00
#include <windows.h>
#include <ProcessSnapshot.h>
#include <iostream>
#include <string>
#include <vector>
#include <stdexcept>
#include <cstdint>
#include <sstream>
#include <map>
struct Handle {
 HANDLE h=nullptr;explicit Handle(HANDLE value=nullptr):h(value){}
 ~Handle(){if(h && h!=INVALID_HANDLE_VALUE)CloseHandle(h);}
 Handle(const Handle&)=delete;Handle& operator=(const Handle&)=delete;
};
struct Error : std::runtime_error {DWORD raw;Error(const char* text,DWORD code):std::runtime_error(text),raw(code){}};
void require(bool yes,const char* text){if(!yes)throw Error(text,GetLastError());}
uint64_t created(HANDLE p){FILETIME c,e,k,u;require(GetProcessTimes(p,&c,&e,&k,&u)!=FALSE,"GetProcessTimes");return (uint64_t(c.dwHighDateTime)<<32)|c.dwLowDateTime;}
std::wstring image(){std::wstring s(32768,L'\0');DWORD n=GetModuleFileNameW(nullptr,s.data(),DWORD(s.size()));require(n && n<s.size(),"self image");s.resize(n);return s;}
std::string line(HANDLE h){std::string value;for(unsigned i=0;i<1024;i++){char c;DWORD got;require(ReadFile(h,&c,1,&got,nullptr)!=FALSE && got==1,"bounded fixture response");if(c=='\n')return value;if(c!='\r')value+=c;}throw Error("fixture line bound",ERROR_INVALID_DATA);}
DWORD elevation(){HANDLE t=nullptr;require(OpenProcessToken(GetCurrentProcess(),TOKEN_QUERY,&t)!=FALSE,"own token");Handle token(t);TOKEN_ELEVATION e{};DWORD n=0;require(GetTokenInformation(t,TokenElevation,&e,sizeof e,&n)!=FALSE,"elevation");return e.TokenIsElevated;}
int broker(){
 require(elevation()==0,"ordinary fixture token required");
 Handle job(CreateJobObjectW(nullptr,nullptr));require(job.h!=nullptr,"own anonymous job");
 auto self=image();std::wstring cmd=L"\""+self+L"\" --owned-never-resumed-launcher";
 STARTUPINFOW start{};start.cb=sizeof start;PROCESS_INFORMATION p{};
 require(CreateProcessW(self.c_str(),cmd.data(),nullptr,nullptr,FALSE,CREATE_SUSPENDED|CREATE_NO_WINDOW,nullptr,nullptr,&start,&p)!=FALSE,"own suspended launcher");
 Handle process(p.hProcess),thread(p.hThread);bool assigned=false;
 try{
  require(AssignProcessToJobObject(job.h,p.hProcess)!=FALSE,"own Job assignment");assigned=true;
  HANDLE duplicate=nullptr;require(DuplicateHandle(GetCurrentProcess(),job.h,GetCurrentProcess(),&duplicate,JOB_OBJECT_QUERY,FALSE,0)!=FALSE,"own duplicate Job QUERY");Handle duplicate_query(duplicate);
  JOBOBJECT_BASIC_ACCOUNTING_INFORMATION a{};require(QueryInformationJobObject(job.h,JobObjectBasicAccountingInformation,&a,sizeof a,nullptr)!=FALSE,"own Job baseline");
  require(a.TotalProcesses==1 && a.ActiveProcesses==1 && a.TotalTerminatedProcesses==0,"own never-resumed Job baseline differs");
  std::cout<<GetCurrentProcessId()<<" "<<p.dwProcessId<<" "<<created(p.hProcess)<<" "<<created(GetCurrentProcess())<<" "<<uintptr_t(job.h)<<" "<<uintptr_t(duplicate)<<std::endl;
  char command=0;DWORD read=0;require(ReadFile(GetStdHandle(STD_INPUT_HANDLE),&command,1,&read,nullptr)!=FALSE && read==1 && command=='x',"explicit own fixture cleanup command");
  // Only this fixture's original Job may terminate its original synthetic child.
  require(TerminateJobObject(job.h,242)!=FALSE,"own fixture cleanup Job");
  require(WaitForSingleObject(p.hProcess,5000)==WAIT_OBJECT_0,"own child exit wait");
  DWORD raw=0;require(GetExitCodeProcess(p.hProcess,&raw)!=FALSE && raw==242,"own child raw242");
  require(QueryInformationJobObject(job.h,JobObjectBasicAccountingInformation,&a,sizeof a,nullptr)!=FALSE && a.ActiveProcesses==0,"own empty Job");
  std::cout<<"cleaned "<<raw<<" "<<a.TotalProcesses<<" "<<a.ActiveProcesses<<std::endl;
  return 0;
 }catch(...){
  // Separate failure cleanup of this own original creation, not a successful
  // observation, recovery proof or action against a caller-selected PID.
  if(assigned)TerminateJobObject(job.h,242);else TerminateProcess(p.hProcess,241);
  WaitForSingleObject(p.hProcess,5000);throw;
 }
}
struct Snapshot {
 HPSS h=nullptr;HANDLE process;Snapshot(HANDLE p,DWORD flags):process(p){DWORD raw=PssCaptureSnapshot(p,PSS_CAPTURE_FLAGS(flags),0,&h);if(raw)throw Error("PssCaptureSnapshot",raw);}
 ~Snapshot(){if(h)PssFreeSnapshot(GetCurrentProcess(),h);}
};
struct Walk {HPSSWALK h=nullptr;Walk(){DWORD raw=PssWalkMarkerCreate(nullptr,&h);if(raw)throw Error("PssWalkMarkerCreate",raw);}~Walk(){if(h)PssWalkMarkerFree(h);}};
BOOL compare_objects(HANDLE a,HANDLE b){
 using Fn=BOOL(WINAPI*)(HANDLE,HANDLE);
 HMODULE module=GetModuleHandleW(L"kernelbase.dll");require(module!=nullptr,"CompareObjectHandles module unavailable");
 auto fn=reinterpret_cast<Fn>(GetProcAddress(module,"CompareObjectHandles"));require(fn!=nullptr,"CompareObjectHandles API unavailable");return fn(a,b);
}
void observe(DWORD pid,uint64_t filetime,DWORD node,uint64_t nodeTime,uintptr_t knownJob,uintptr_t knownDuplicate,DWORD rights,DWORD flags){
 std::cout<<"{\"rights\":"<<rights<<",\"captureFlags\":"<<flags;
 try{
  const auto deadline=GetTickCount64()+5000;
  Handle source(OpenProcess(rights,FALSE,pid));require(source.h!=nullptr,"minimum broker OpenProcess");
  require(created(source.h)==filetime && WaitForSingleObject(source.h,0)==WAIT_TIMEOUT,"original owned broker generation");
  Handle target(OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION|SYNCHRONIZE,FALSE,node));require(target.h!=nullptr,"own launcher query");require(created(target.h)==nodeTime,"own launcher generation");
  Snapshot snapshot(source.h,flags);require(GetTickCount64()<deadline,"snapshot original5s observation expired");Walk walk;
  PSS_HANDLE_INFORMATION captured{};DWORD raw=PssQuerySnapshot(snapshot.h,PSS_QUERY_HANDLE_INFORMATION,&captured,sizeof captured);if(raw)throw Error("PssQuerySnapshot(handles)",raw);
  require(captured.HandlesCaptured>0 && captured.HandlesCaptured<=20000,"bounded handle count");
  unsigned count=0,withType=0,missingType=0,jobs=0,jobDuplicates=0,matching=0,ownJobEntries=0,ownJobTypeMissing=0,ownJobTypeLength=0,ownJobTypeEnd=0;std::vector<HANDLE> queries;std::map<std::pair<unsigned,unsigned>,unsigned> missingTypes;
  try{
   for(;;){
    PSS_HANDLE_ENTRY entry{};raw=PssWalkSnapshot(snapshot.h,PSS_WALK_HANDLES,walk.h,&entry,sizeof entry);
    if(raw==ERROR_NO_MORE_ITEMS)break;if(raw)throw Error("PssWalkSnapshot",raw);
    require(GetTickCount64()<deadline,"handle walk original5s observation expired");
    require(++count<=20000,"walk count bound");
    // These two handle integers come only from our own fixture and are
    // diagnostic locators, never candidate duplication/Job authority.
    if(uintptr_t(entry.Handle)==knownJob || uintptr_t(entry.Handle)==knownDuplicate){ownJobEntries++;if(!entry.TypeName || !entry.TypeNameLength)ownJobTypeMissing++;else{ownJobTypeLength=entry.TypeNameLength;if(entry.TypeNameLength%sizeof(wchar_t)==0 && entry.TypeNameLength<=512)ownJobTypeEnd=entry.TypeName[entry.TypeNameLength/sizeof(wchar_t)-1];}}
    // Never dereference/log ObjectName or unrelated file/token contents.
    if(!entry.TypeName || entry.TypeNameLength==0){missingType++;missingTypes[{unsigned(entry.ObjectType),unsigned(entry.Flags)}]++;continue;}
    require(entry.TypeNameLength%sizeof(wchar_t)==0 && entry.TypeNameLength<=512,"type name byte bound");withType++;
    std::wstring type(entry.TypeName,entry.TypeNameLength/sizeof(wchar_t));
    if(!type.empty() && type.back()==L'\0')type.pop_back();
    require(type.find(L'\0')==std::wstring::npos,"embedded-null object type refused");
    if(type!=L"Job")continue;jobs++;
    HANDLE copy=nullptr;require(DuplicateHandle(source.h,entry.Handle,GetCurrentProcess(),&copy,JOB_OBJECT_QUERY,FALSE,0)!=FALSE,"Job QUERY duplicate only");
    bool duplicate=false;for(HANDLE old:queries){SetLastError(ERROR_SUCCESS);BOOL same=compare_objects(old,copy);if(same){duplicate=true;break;}require(GetLastError()==ERROR_SUCCESS || GetLastError()==ERROR_NOT_SAME_OBJECT,"CompareObjectHandles unknown");}
    if(duplicate){jobDuplicates++;CloseHandle(copy);continue;}queries.push_back(copy);
    BOOL inJob=FALSE;require(IsProcessInJob(target.h,copy,&inJob)!=FALSE,"actual original launcher Job membership");if(inJob)matching++;
   }
   require(count==captured.HandlesCaptured,"complete walk/captured count");
   bool identifiable=missingType==0 && jobs>0 && matching==1;
   std::cout<<",\"captureRaw\":0,\"walkEndRaw\":"<<raw<<",\"captured\":"<<captured.HandlesCaptured<<",\"walked\":"<<count<<",\"entriesWithType\":"<<withType<<",\"missingType\":"<<missingType<<",\"ownFixtureJobEntries\":"<<ownJobEntries<<",\"ownFixtureJobMissingType\":"<<ownJobTypeMissing<<",\"ownFixtureJobTypeByteLength\":"<<ownJobTypeLength<<",\"ownFixtureJobTypeLastCodeUnit\":"<<ownJobTypeEnd<<",\"jobCandidates\":"<<jobs<<",\"duplicateJobHandles\":"<<jobDuplicates<<",\"distinctJobs\":"<<queries.size()<<",\"matchingJobs\":"<<matching<<",\"identifiableInThisSinglePass\":"<<(identifiable?"true":"false")<<",\"accepted\":false,\"missingTypeKinds\":[";bool first=true;for(const auto& row:missingTypes){if(!first)std::cout<<",";first=false;std::cout<<"{\"objectType\":"<<row.first.first<<",\"flags\":"<<row.first.second<<",\"count\":"<<row.second<<"}";}std::cout<<"],\"scope\":\"capability only; no two-pass drift/accounting/rejection matrix yet\"}"<<std::endl;
  }catch(...){for(HANDLE h:queries)CloseHandle(h);throw;}
  for(HANDLE h:queries)CloseHandle(h);
  require(created(source.h)==filetime && WaitForSingleObject(source.h,0)==WAIT_TIMEOUT,"broker changed after snapshot");
 }catch(const Error& e){std::cout<<",\"accepted\":false,\"error\":\""<<e.what()<<"\",\"raw\":"<<e.raw<<"}"<<std::endl;}
}
int main(int argc,char** argv){
 try{
  if(argc==2 && std::string(argv[1])=="--owned-broker")return broker();
  if(argc==2 && std::string(argv[1])=="--owned-never-resumed-launcher")return 73;
  require(argc==1,"no caller target or arguments allowed");require(elevation()==0,"non-elevated observer required");
  SECURITY_ATTRIBUTES sa{sizeof(sa),nullptr,TRUE};HANDLE inRead=nullptr,inWrite=nullptr,outRead=nullptr,outWrite=nullptr;
  require(CreatePipe(&inRead,&inWrite,&sa,0)!=FALSE,"own stdin pipe");Handle ir(inRead),iw(inWrite);
  require(CreatePipe(&outRead,&outWrite,&sa,0)!=FALSE,"own stdout pipe");Handle ore(outRead),ow(outWrite);
  require(SetHandleInformation(inWrite,HANDLE_FLAG_INHERIT,0)!=FALSE && SetHandleInformation(outRead,HANDLE_FLAG_INHERIT,0)!=FALSE,"no parent endpoint inheritance");
  auto self=image();std::wstring cmd=L"\""+self+L"\" --owned-broker";STARTUPINFOW start{};start.cb=sizeof start;start.dwFlags=STARTF_USESTDHANDLES;start.hStdInput=inRead;start.hStdOutput=outWrite;start.hStdError=outWrite;
  PROCESS_INFORMATION p{};require(CreateProcessW(self.c_str(),cmd.data(),nullptr,nullptr,TRUE,CREATE_NO_WINDOW,nullptr,nullptr,&start,&p)!=FALSE,"own broker creation");Handle process(p.hProcess),thread(p.hThread);
  DWORD brokerPid=0,nodePid=0;uint64_t nodeTime=0,brokerTime=0;uintptr_t ownJob=0,ownDuplicate=0;std::string text=line(outRead);std::istringstream input(text);input>>brokerPid>>nodePid>>nodeTime>>brokerTime>>ownJob>>ownDuplicate;
  require(brokerPid==p.dwProcessId && brokerTime==created(p.hProcess) && nodePid && nodeTime && ownJob && ownDuplicate && ownJob!=ownDuplicate,"own original broker receipt");
  for(DWORD rights:{DWORD(PROCESS_QUERY_LIMITED_INFORMATION|SYNCHRONIZE),DWORD(PROCESS_QUERY_LIMITED_INFORMATION|PROCESS_DUP_HANDLE|SYNCHRONIZE),DWORD(PROCESS_QUERY_INFORMATION|PROCESS_DUP_HANDLE|SYNCHRONIZE)})
   for(DWORD flags:{DWORD(PSS_CAPTURE_HANDLES),DWORD(PSS_CAPTURE_HANDLES|PSS_CAPTURE_HANDLE_BASIC_INFORMATION),DWORD(PSS_CAPTURE_HANDLES|PSS_CAPTURE_HANDLE_BASIC_INFORMATION|PSS_CAPTURE_HANDLE_TYPE_SPECIFIC_INFORMATION)})
    observe(brokerPid,brokerTime,nodePid,nodeTime,ownJob,ownDuplicate,rights,flags);
  char command='x';DWORD written=0;require(WriteFile(inWrite,&command,1,&written,nullptr)!=FALSE && written==1,"own cleanup signal");
  auto cleanup=line(outRead);require(cleanup=="cleaned 242 1 0","own cleanup receipt");require(WaitForSingleObject(p.hProcess,5000)==WAIT_OBJECT_0,"own broker normal exit");DWORD raw=0;require(GetExitCodeProcess(p.hProcess,&raw)!=FALSE && raw==0,"own broker raw0");
  std::cout<<"{\"probeFinished\":true,\"ownLauncherRaw\":242,\"ownBrokerRaw\":0,\"currentR2Touched\":false,\"objectNamesRead\":false,\"memoryCaptured\":false,\"accepted\":false}"<<std::endl;
  return 0;
 }catch(const Error& e){std::cerr<<"probe FAILED "<<e.what()<<" raw="<<e.raw<<std::endl;return 1;}
}
