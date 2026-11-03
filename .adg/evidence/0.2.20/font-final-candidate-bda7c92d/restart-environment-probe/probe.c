#include <stdlib.h>
#include <stdio.h>
#include <unistd.h>
int main(void) {
  FILE *f=fopen("/tmp/ocx020-macos-restart-environment-probe/observed.txt","w");
  if(!f)return 2;
  const char *a=getenv("OPENCODEX_SANDBOX"),*b=getenv("OPENCODEX_SANDBOX_ROOT");
  fprintf(f,"pid=%d sandbox=%s root=%s",getpid(),a?a:"absent",b?b:"absent");
  return fclose(f);
}
