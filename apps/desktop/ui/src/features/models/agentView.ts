import type {Proposal,ResearchSource,TemplateDraft} from './api'
/** Session-only view state. No credentials, execution preference or portable policy. */
export interface AgentViewContext {initialized:boolean;lockContext?:boolean;sourcesOpen?:boolean;input:string;urls:string;query:string;history:{role:string;text:string}[];sources:ResearchSource[];scrollTop:number;selectionStart:number;selectionEnd:number;focusInput:boolean;base:TemplateDraft|null;proposal:Proposal|null;chosen:string[];maximized:boolean}
export const createAgentViewContext=():AgentViewContext=>({initialized:false,lockContext:true,sourcesOpen:false,input:'',urls:'',query:'',history:[],sources:[],scrollTop:0,selectionStart:0,selectionEnd:0,focusInput:false,base:null,proposal:null,chosen:[],maximized:false})
