// TEAM24/r7 documented ObjectTypeInformation capability and boundary probe.
// Only own broker inputs; no caller target, no names/data/private NT layouts.
#define main previous_capability_main
#include "r7-pss-job-probe.cpp"
#undef main
#include <winternl.h>
#include <algorithm>
#include <cstring>
#include <limits>
#include <memory>
#include <cstddef>
using QueryObject=NTSTATUS(NTAPI*)(HANDLE,OBJECT_INFORMATION_CLASS,PVOID,ULONG,PULONG);
struct Bytes {
 std::vector<std::max_align_t> storage;size_t size;
 explicit Bytes(size_t n):storage((n+sizeof(std::max_align_t)-1)/sizeof(std::max_align_t)),size(n){}
 void* data(){return storage.data();}
};
void within_type(ULONGLONG end){if(GetTickCount64()>=end)throw Error("original type query cutoff expired",ERROR_TIMEOUT);}
std::wstring parse_public_type(void* data,size_t capacity,ULONG returned){
 // Only the documented TypeName field is read; Reserved is never accessed.
 if(returned<sizeof(PUBLIC_OBJECT_TYPE_INFORMATION) || returned>capacity)throw Error("public type return length",ERROR_INVALID_DATA);
 const auto& name=static_cast<PUBLIC_OBJECT_TYPE_INFORMATION*>(data)->TypeName;
 const uintptr_t base=reinterpret_cast<uintptr_t>(data),pointer=reinterpret_cast<uintptr_t>(name.Buffer);
 if(name.Length==0 || name.Length>512 || name.Length%2 || name.MaximumLength%2
    || name.MaximumLength<name.Length || !name.Buffer || pointer%alignof(wchar_t)
    || returned>std::numeric_limits<uintptr_t>::max()-base || pointer<base+sizeof(PUBLIC_OBJECT_TYPE_INFORMATION)
    || pointer>base+returned || name.MaximumLength>base+returned-pointer)throw Error("bounded public TypeName pointer/length",ERROR_INVALID_DATA);
 const size_t count=name.Length/sizeof(wchar_t);
 std::wstring value;value.reserve(count);
 for(size_t i=0;i<count;i++){
  wchar_t c=name.Buffer[i];
  if(c==0)throw Error("embedded null public type",ERROR_INVALID_DATA);
  if(c>=0xd800 && c<=0xdbff){if(++i>=count || name.Buffer[i]<0xdc00 || name.Buffer[i]>0xdfff)throw Error("unpaired high surrogate",ERROR_INVALID_DATA);value+=c;value+=name.Buffer[i];}
  else if(c>=0xdc00 && c<=0xdfff)throw Error("unpaired low surrogate",ERROR_INVALID_DATA);
  else value+=c;
 }
 return value;
}
std::wstring query_type(HANDLE copy,ULONGLONG end){
 within_type(end);auto module=GetModuleHandleW(L"ntdll.dll");
 auto query=reinterpret_cast<QueryObject>(module?GetProcAddress(module,"NtQueryObject"):nullptr);
 if(!query)throw Error("documented NtQueryObject export missing",ERROR_PROC_NOT_FOUND);
 Bytes first(4096);ULONG returned=0;
 NTSTATUS status=query(copy,ObjectTypeInformation,first.data(),ULONG(first.size),&returned);
 within_type(end);
 if(status==0)return parse_public_type(first.data(),first.size,returned);
 const auto raw=static_cast<ULONG>(status);
 if((raw!=0xc0000004UL && raw!=0xc0000023UL && raw!=0x80000005UL)
    || returned<=first.size || returned>65536)throw Error("NtQueryObject type first status",raw);
 Bytes second(returned);returned=0;
 status=query(copy,ObjectTypeInformation,second.data(),ULONG(second.size),&returned);
 within_type(end);if(status!=0)throw Error("NtQueryObject type resize status",static_cast<ULONG>(status));
 return parse_public_type(second.data(),second.size,returned);
}
struct Copied {
 HANDLE source=nullptr;Handle zero;std::wstring type;
 Copied(HANDLE s,HANDLE z,std::wstring t):source(s),zero(z),type(std::move(t)){}
};
struct Pass {
 std::vector<std::unique_ptr<Copied>> entries;
 std::map<std::wstring,unsigned> types;std::map<std::pair<std::string,DWORD>,unsigned> errors;
 ULONG captured=0,walked=0,endRaw=0;unsigned unknown=0;
};
Pass typed_pass(HANDLE broker,ULONGLONG end){
 within_type(end);Pass out;Snapshot snapshot(broker,PSS_CAPTURE_HANDLES);Walk marker;
 PSS_HANDLE_INFORMATION count{};auto raw=PssQuerySnapshot(snapshot.h,PSS_QUERY_HANDLE_INFORMATION,&count,sizeof count);
 if(raw)throw Error("PSS type count",raw);
 require(count.HandlesCaptured>0 && count.HandlesCaptured<=20000,"bounded complete original handle count");out.captured=count.HandlesCaptured;
 for(;;){
  within_type(end);PSS_HANDLE_ENTRY entry{};raw=PssWalkSnapshot(snapshot.h,PSS_WALK_HANDLES,marker.h,&entry,sizeof entry);
  if(raw==ERROR_NO_MORE_ITEMS){out.endRaw=raw;break;}if(raw)throw Error("PSS type walk",raw);
  require(++out.walked<=20000,"type walk count bound");
  try{
   HANDLE local=nullptr;
   if(!DuplicateHandle(broker,entry.Handle,GetCurrentProcess(),&local,0,FALSE,0))throw Error("zero-rights duplicate",GetLastError());
   auto copy=std::make_unique<Copied>(entry.Handle,local,L"");
   copy->type=query_type(local,end);out.types[copy->type]++;out.entries.push_back(std::move(copy));
  }catch(const Error& e){out.unknown++;out.errors[{e.what(),e.raw}]++;}
 }
 require(out.captured==out.walked,"complete captured/walked coverage");within_type(end);return out;
}
void print_pass(const Pass& p,int pass){
 std::cout<<"{\"pass\":"<<pass<<",\"captureFlags\":4,\"captured\":"<<p.captured<<",\"walked\":"<<p.walked<<",\"walkEndRaw\":"<<p.endRaw<<",\"typed\":"<<p.entries.size()<<",\"unknown\":"<<p.unknown<<",\"types\":[";bool first=true;
 for(const auto& [name,count]:p.types){if(!first)std::cout<<",";first=false;
  // Type names alone: no ObjectName/addresses/source integers are output.
  std::string ascii;for(wchar_t c:name){if(c<32 || c>126 || c=='\"' || c=='\\')throw Error("type aggregate escaping unknown",ERROR_INVALID_DATA);ascii+=char(c);}
  std::cout<<"{\"type\":\""<<ascii<<"\",\"count\":"<<count<<"}";
 }
 std::cout<<"],\"errors\":[";first=true;for(const auto& [key,count]:p.errors){if(!first)std::cout<<",";first=false;std::cout<<"{\"stage\":\""<<key.first<<"\",\"raw\":"<<key.second<<",\"count\":"<<count<<"}";}
 std::cout<<"],\"accepted\":false,\"currentR2Touched\":false}"<<std::endl;
}
void buffer_negatives(){
 const auto header=sizeof(PUBLIC_OBJECT_TYPE_INFORMATION);Bytes raw(header+32);
 auto* info=static_cast<PUBLIC_OBJECT_TYPE_INFORMATION*>(raw.data());
 auto reset=[&](){std::memset(raw.data(),0,raw.size);info->TypeName.Length=6;info->TypeName.MaximumLength=8;info->TypeName.Buffer=reinterpret_cast<PWSTR>(static_cast<unsigned char*>(raw.data())+header);std::memcpy(info->TypeName.Buffer,L"Job",8);};
 reset();require(parse_public_type(raw.data(),raw.size,ULONG(raw.size))==L"Job","valid public buffer fixture");
 unsigned rejected=0;for(unsigned i=0;i<12;i++){reset();ULONG returned=ULONG(raw.size);
  switch(i){case 0:returned=ULONG(header-1);break;case 1:returned=ULONG(raw.size+1);break;case 2:info->TypeName.Length=0;break;case 3:info->TypeName.Length=3;break;case 4:info->TypeName.Length=514;break;case 5:info->TypeName.MaximumLength=4;break;case 6:info->TypeName.Buffer=nullptr;break;case 7:info->TypeName.Buffer=reinterpret_cast<PWSTR>(static_cast<unsigned char*>(raw.data())+2);break;case 8:info->TypeName.Buffer=reinterpret_cast<PWSTR>(static_cast<unsigned char*>(raw.data())+raw.size-2);break;case 9:info->TypeName.Buffer[0]=0xd800;break;case 10:info->TypeName.Buffer[0]=0xdc00;break;case 11:info->TypeName.Buffer[1]=0;break;}
  bool refused=false;try{parse_public_type(raw.data(),raw.size,returned);}catch(const Error&){refused=true;}require(refused,"malformed public type buffer accepted");rejected++;
 }
 bool expired=false;try{within_type(GetTickCount64());}catch(const Error&){expired=true;}require(expired,"type expiry refused");
 std::cout<<"{\"publicBufferNegativesPassed\":"<<rejected<<",\"expiredDeadlineRefused\":true,\"nativeAuthority\":false}"<<std::endl;
}
void observe_types(DWORD pid,uint64_t brokerTime,DWORD node,uint64_t nodeTime){
 const auto end=GetTickCount64()+5000;
 Handle source(OpenProcess(PROCESS_QUERY_INFORMATION|PROCESS_DUP_HANDLE|SYNCHRONIZE,FALSE,pid));require(source.h!=nullptr,"own type broker query");
 require(created(source.h)==brokerTime && WaitForSingleObject(source.h,0)==WAIT_TIMEOUT,"original own broker identity");
 Handle target(OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION|SYNCHRONIZE,FALSE,node));require(target.h!=nullptr && created(target.h)==nodeTime,"own node identity");
 auto first=typed_pass(source.h,end);print_pass(first,1);auto second=typed_pass(source.h,end);print_pass(second,2);
 bool stable=first.unknown==0 && second.unknown==0 && first.entries.size()==second.entries.size();
 if(stable){
  for(const auto& a:first.entries){auto it=std::find_if(second.entries.begin(),second.entries.end(),[&](const auto& b){return b->source==a->source;});
   if(it==second.entries.end() || (*it)->type!=a->type || !compare_objects(a->zero.h,(*it)->zero.h)){stable=false;break;}
  }
 }
 unsigned jobs=0,duplicates=0,matching=0;std::vector<std::unique_ptr<Handle>> queries;
 if(stable){for(const auto& entry:first.entries){within_type(end);if(entry->type!=L"Job")continue;
  HANDLE q=nullptr;if(!DuplicateHandle(source.h,entry->source,GetCurrentProcess(),&q,JOB_OBJECT_QUERY,FALSE,0))throw Error("same original Job QUERY copy",GetLastError());auto job=std::make_unique<Handle>(q);
  require(compare_objects(entry->zero.h,q)!=FALSE,"zero/query same original Job kernel object");jobs++;
  if(std::any_of(queries.begin(),queries.end(),[&](const auto& old){return compare_objects(old->h,q)!=FALSE;})){duplicates++;continue;}
  BOOL member=FALSE;require(IsProcessInJob(target.h,q,&member)!=FALSE,"original node membership");if(member)matching++;
  JOBOBJECT_BASIC_ACCOUNTING_INFORMATION accounting{};require(QueryInformationJobObject(q,JobObjectBasicAccountingInformation,&accounting,sizeof accounting,nullptr)!=FALSE,"original Job historical accounting");
  std::cout<<"{\"jobTotal\":"<<accounting.TotalProcesses<<",\"jobActive\":"<<accounting.ActiveProcesses<<",\"jobTerminated\":"<<accounting.TotalTerminatedProcesses<<",\"nodeMember\":"<<(member?"true":"false")<<",\"accepted\":false}"<<std::endl;queries.push_back(std::move(job));
 }}
 require(created(source.h)==brokerTime && WaitForSingleObject(source.h,0)==WAIT_TIMEOUT && created(target.h)==nodeTime,"original source and node stable after two type passes");
 within_type(end);
 std::cout<<"{\"completeTypeCoverage\":"<<(first.unknown==0 && second.unknown==0?"true":"false")<<",\"twoPassObjectsStable\":"<<(stable?"true":"false")<<",\"jobCandidates\":"<<jobs<<",\"duplicateJobs\":"<<duplicates<<",\"matchingJobs\":"<<matching<<",\"fullPrototypeRejectionMatrixComplete\":false,\"accepted\":false,\"currentR2Touched\":false}"<<std::endl;
}
int main(int argc,char** argv){
 try{
  if(argc==2 && std::string(argv[1])=="--owned-broker")return broker();
  if(argc==2 && std::string(argv[1])=="--owned-never-resumed-launcher")return 73;
  require(argc==1,"no external target arguments");require(elevation()==0,"ordinary token type observer");buffer_negatives();
  SECURITY_ATTRIBUTES sa{sizeof(sa),nullptr,TRUE};HANDLE inRead=nullptr,inWrite=nullptr,outRead=nullptr,outWrite=nullptr;
  require(CreatePipe(&inRead,&inWrite,&sa,0)!=FALSE,"own input pipe");Handle ir(inRead),iw(inWrite);
  require(CreatePipe(&outRead,&outWrite,&sa,0)!=FALSE,"own output pipe");Handle ore(outRead),ow(outWrite);
  require(SetHandleInformation(inWrite,HANDLE_FLAG_INHERIT,0)!=FALSE && SetHandleInformation(outRead,HANDLE_FLAG_INHERIT,0)!=FALSE,"own endpoint inheritance");
  auto self=image();std::wstring cmd=L"\""+self+L"\" --owned-broker";STARTUPINFOW start{};start.cb=sizeof start;start.dwFlags=STARTF_USESTDHANDLES;start.hStdInput=inRead;start.hStdOutput=outWrite;start.hStdError=outWrite;PROCESS_INFORMATION p{};
  require(CreateProcessW(self.c_str(),cmd.data(),nullptr,nullptr,TRUE,CREATE_NO_WINDOW,nullptr,nullptr,&start,&p)!=FALSE,"own type broker");Handle process(p.hProcess),thread(p.hThread);
  DWORD pid=0,node=0;uint64_t nodeTime=0,brokerTime=0;uintptr_t job=0,duplicate=0;std::istringstream input(line(outRead));input>>pid>>node>>nodeTime>>brokerTime>>job>>duplicate;
  require(pid==p.dwProcessId && brokerTime==created(p.hProcess) && node && nodeTime,"exact own original broker receipt");
  try{observe_types(pid,brokerTime,node,nodeTime);}catch(const Error& e){std::cout<<"{\"observeError\":\""<<e.what()<<"\",\"raw\":"<<e.raw<<",\"accepted\":false,\"currentR2Touched\":false}"<<std::endl;}
  char command='x';DWORD written=0;require(WriteFile(inWrite,&command,1,&written,nullptr)!=FALSE && written==1,"own fixture cleanup only");require(line(outRead)=="cleaned 242 1 0","own native cleanup receipt");
  require(WaitForSingleObject(p.hProcess,5000)==WAIT_OBJECT_0,"own broker exit");DWORD raw=0;require(GetExitCodeProcess(p.hProcess,&raw)!=FALSE && raw==0,"own broker raw0");
  std::cout<<"{\"finished\":true,\"ownLauncherRaw\":242,\"ownBrokerRaw\":0,\"zeroDesiredAccess\":true,\"objectNamesRead\":false,\"reservedRead\":false,\"snapshotPersisted\":false,\"currentR2Touched\":false,\"accepted\":false}"<<std::endl;return 0;
 }catch(const Error& e){std::cerr<<"type probe FAILED "<<e.what()<<" raw="<<e.raw<<std::endl;return 1;}
}
