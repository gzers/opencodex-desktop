using System;
using System.Diagnostics;
using System.IO;
using System.Windows.Forms;
// Real isolated acceptance shell, never the daily Codex desktop installation.
class OwnedClient {
 [STAThread] static void Main(string[] args) {
  string root=Path.GetFullPath(args[0]);
  if(!root.Contains("post021-restart-panel"))throw new Exception("isolated root required");
  string value=File.ReadAllText(Path.Combine(root,"home",".codex","config.toml"));
  File.WriteAllText(Path.Combine(root,"client-readback-"+Process.GetCurrentProcess().Id+".txt"),value);
  Application.EnableVisualStyles();
  Form form=new Form();form.Text="Windows restart acceptance (isolated)";form.Width=760;form.Height=360;
  TextBox text=new TextBox();text.Multiline=true;text.ReadOnly=true;text.Dock=DockStyle.Fill;
  text.Text="Isolated acceptance client process "+Process.GetCurrentProcess().Id+Environment.NewLine+value;
  form.Controls.Add(text);Application.Run(form);
 }
}
