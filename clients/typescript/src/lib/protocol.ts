export interface TransferCompleteMessage {
  id: string;
}

export interface TransferErrorMessage {
  id: string;
  response: ErrorResponse;
}

export interface AccountInitialization {
  currency: Currency;
  initial_position: Amount;
  ndc: number;
}

export interface ClientParticipant {
  name: FspId;
  account: AccountInitialization;
}

export interface TransferMessage {
  msg_sender: FspId;
  msg_recipient: FspId;
  currency: Currency;
  amount: Amount;
  transfer_id: string;
}

export interface SettlementModelCreatedMessage {
  settlement_model: SettlementModel;
}

export interface SettlementWindowCloseMessage {
  id: SettlementWindowId;
  reason: string;
}

export interface SettlementWindowCloseFailedMessage {
  id: SettlementWindowId;
  response: ErrorResponse;
}

export type SettlementWindowState =
  | "OPEN"
  | "CLOSED"
  | "PENDING_SETTLEMENT"
  | "SETTLED"
  | "ABORTED";

export interface SettlementWindowContent {
  id: SettlementWindowContentId;
  settlementWindowId: SettlementWindowId | null;
  state: SettlementWindowState;
  ledgerAccountType: LedgerAccountType;
  currencyId: Currency;
  createdDate: DateTime;
  changedDate: DateTime | null;
  settlementId: SettlementId | null;
}

export interface SettlementWindow {
  settlementWindowId: SettlementWindowId;
  reason: string | null;
  state: SettlementWindowState;
  createdDate: DateTime;
  changedDate: DateTime | null;
  content: SettlementWindowContent[] | null;
}

export type SettlementWindowId = number;

export type SettlementWindowContentId = number;

export interface GetSettlementWindows {
  currency: Currency | null;
  participantId: FspId | null;
  state: SettlementWindowState | null;
  fromDateTime: DateTime | null;
  toDateTime: DateTime | null;
}

export type SettlementAccountType = "SETTLEMENT" | "INTERCHANGE_FEE_SETTLEMENT";

export type SettlementDelay = "DEFERRED" | "IMMEDIATE";

export type SettlementGranularity = "GROSS" | "NET";

export type SettlementInterchange = "BILATERAL" | "MULTILATERAL";

export type LedgerAccountType = "INTERCHANGE_FEE" | "POSITION";

export interface SettlementModel {
  autoPositionReset: boolean;
  ledgerAccountType: LedgerAccountType;
  settlementAccountType: SettlementAccountType;
  name: string;
  requireLiquidityCheck: boolean;
  settlementDelay: SettlementDelay;
  settlementGranularity: SettlementGranularity;
  settlementInterchange: SettlementInterchange;
  currency: Currency;
}

export interface GetSettlements {
  currency: Currency | null;
  participantId: FspId | null;
  settlementWindowId: SettlementWindowId | null;
  state: SettlementState | null;
  fromDateTime: DateTime | null;
  toDateTime: DateTime | null;
  fromSettlementWindowDateTime: DateTime | null;
  toSettlementWindowDateTime: DateTime | null;
}

export interface NewSettlement {
  settlementModel: string;
  reason: string;
  settlementWindows: WindowParametersNewSettlement[];
}

export interface SettlementSettlementWindow {
  id: SettlementWindowId;
  reason: string | null;
  state: SettlementWindowState;
  createdDate: DateTime;
  changedDate: DateTime | null;
  content: SettlementWindowContent[] | null;
}

export interface Settlement {
  id: SettlementId;
  state: SettlementState;
  createdDate: DateTime;
  changedDate: DateTime;
  settlementWindows: SettlementSettlementWindow[];
  participants: SettlementParticipant[];
}

export type SettlementId = number;

export type SettlementState =
  | "PENDING_SETTLEMENT"
  | "PS_TRANSFERS_RECORDED"
  | "PS_TRANSFERS_RESERVED"
  | "PS_TRANSFERS_COMMITTED"
  | "SETTLING"
  | "SETTLED"
  | "ABORTED";

export interface SettlementParticipant {
  id: ParticipantId;
  accounts: SettlementAccount[];
}

export interface SettlementAccount {
  id: ParticipantCurrencyId;
  reason: string;
  state: SettlementState;
  netSettlementAmount: NetSettlementAmount;
}

export type ParticipantId = number;

export type ParticipantCurrencyId = number;

export interface NetSettlementAmount {
  amount: Amount;
  currency: Currency;
}

export interface WindowParametersNewSettlement {
  id: SettlementWindowId;
}

export interface HubAccount {
  type: HubAccountType;
  currency: Currency;
}

export type HubAccountType =
  | "HUB_MULTILATERAL_SETTLEMENT"
  | "HUB_RECONCILIATION";

export type FspId = string;

export type DateTime = string;

export type Amount = string;

export type Currency =
  | "AED"
  | "AFA"
  | "AFN"
  | "ALL"
  | "AMD"
  | "ANG"
  | "AOA"
  | "AOR"
  | "ARS"
  | "AUD"
  | "AWG"
  | "AZN"
  | "BAM"
  | "BBD"
  | "BDT"
  | "BGN"
  | "BHD"
  | "BIF"
  | "BMD"
  | "BND"
  | "BOB"
  | "BOV"
  | "BRL"
  | "BSD"
  | "BTN"
  | "BWP"
  | "BYN"
  | "BYR"
  | "BZD"
  | "CAD"
  | "CDF"
  | "CHE"
  | "CHF"
  | "CHW"
  | "CLF"
  | "CLP"
  | "CNY"
  | "COP"
  | "COU"
  | "CRC"
  | "CUC"
  | "CUP"
  | "CVE"
  | "CZK"
  | "DJF"
  | "DKK"
  | "DOP"
  | "DZD"
  | "EEK"
  | "EGP"
  | "ERN"
  | "ETB"
  | "EUR"
  | "FJD"
  | "FKP"
  | "GBP"
  | "GEL"
  | "GGP"
  | "GHS"
  | "GIP"
  | "GMD"
  | "GNF"
  | "GTQ"
  | "GYD"
  | "HKD"
  | "HNL"
  | "HRK"
  | "HTG"
  | "HUF"
  | "IDR"
  | "ILS"
  | "IMP"
  | "INR"
  | "IQD"
  | "IRR"
  | "ISK"
  | "JEP"
  | "JMD"
  | "JOD"
  | "JPY"
  | "KES"
  | "KGS"
  | "KHR"
  | "KMF"
  | "KPW"
  | "KRW"
  | "KWD"
  | "KYD"
  | "KZT"
  | "LAK"
  | "LBP"
  | "LKR"
  | "LRD"
  | "LSL"
  | "LTL"
  | "LVL"
  | "LYD"
  | "MAD"
  | "MDL"
  | "MGA"
  | "MKD"
  | "MMK"
  | "MNT"
  | "MOP"
  | "MRO"
  | "MUR"
  | "MVR"
  | "MWK"
  | "MXN"
  | "MXV"
  | "MYR"
  | "MZN"
  | "NAD"
  | "NGN"
  | "NIO"
  | "NOK"
  | "NPR"
  | "NZD"
  | "OMR"
  | "PAB"
  | "PEN"
  | "PGK"
  | "PHP"
  | "PKR"
  | "PLN"
  | "PYG"
  | "QAR"
  | "RON"
  | "RSD"
  | "RUB"
  | "RWF"
  | "SAR"
  | "SBD"
  | "SCR"
  | "SDG"
  | "SEK"
  | "SGD"
  | "SHP"
  | "SLL"
  | "SOS"
  | "SPL"
  | "SRD"
  | "SSP"
  | "STD"
  | "SVC"
  | "SYP"
  | "SZL"
  | "THB"
  | "TJS"
  | "TMT"
  | "TND"
  | "TOP"
  | "TRY"
  | "TTD"
  | "TVD"
  | "TWD"
  | "TZS"
  | "UAH"
  | "UGX"
  | "USD"
  | "USN"
  | "UYI"
  | "UYU"
  | "UZS"
  | "VEF"
  | "VND"
  | "VUV"
  | "WST"
  | "XAF"
  | "XAG"
  | "XAU"
  | "XCD"
  | "XDR"
  | "XFO"
  | "XFU"
  | "XOF"
  | "XPD"
  | "XPF"
  | "XPT"
  | "XSU"
  | "XTS"
  | "XUA"
  | "XXX"
  | "YER"
  | "ZAR"
  | "ZMK"
  | "ZMW"
  | "ZWD"
  | "ZWL"
  | "ZWN"
  | "ZWR";

export interface ErrorResponse {
  errorInformation: ErrorInformation;
}

export interface ErrorInformation {
  errorCode: MojaloopApiError;
  errorDescription: string;
}

export type MojaloopApiError =
  | "1000"
  | "1001"
  | "2000"
  | "2001"
  | "2002"
  | "2003"
  | "2004"
  | "2005"
  | "3000"
  | "3000"
  | "3001"
  | "3002"
  | "3003"
  | "3040"
  | "3100"
  | "3101"
  | "3102"
  | "3103"
  | "3104"
  | "3105"
  | "3106"
  | "3107"
  | "3200"
  | "3201"
  | "3202"
  | "3203"
  | "3204"
  | "3205"
  | "3206"
  | "3207"
  | "3208"
  | "3209"
  | "3210"
  | "3300"
  | "3301"
  | "3302"
  | "3303"
  | "4000"
  | "4001"
  | "4100"
  | "4101"
  | "4102"
  | "4103"
  | "4200"
  | "4300"
  | "4400"
  | "5000"
  | "5001"
  | "5100"
  | "5101"
  | "5102"
  | "5103"
  | "5104"
  | "5105"
  | "5106"
  | "5200"
  | "5300"
  | "5400";

export type ServerMessage =
  | { type: "TransferComplete"; value: TransferCompleteMessage }
  | { type: "TransferError"; value: TransferErrorMessage }
  | { type: "AssignParticipants"; value: ClientParticipant[] }
  | { type: "HubAccountsCreated"; value: HubAccount[] }
  | { type: "SettlementModelCreated"; value: SettlementModelCreatedMessage }
  | { type: "SettlementWindowClosed"; value: SettlementWindowId }
  | {
    type: "SettlementWindowCloseFailed";
    value: SettlementWindowCloseFailedMessage;
  }
  | { type: "SettlementWindows"; value: SettlementWindow[] }
  | { type: "Settlements"; value: Settlement[] }
  | { type: "NewSettlementCreated"; value: Settlement };

export type ClientMessage =
  | { type: "Transfers"; value: TransferMessage[] }
  | { type: "CreateHubAccounts"; value: HubAccount[] }
  | { type: "CreateParticipants"; value: AccountInitialization[] }
  | { type: "CreateSettlementModel"; value: SettlementModel }
  | { type: "CloseSettlementWindow"; value: SettlementWindowCloseMessage }
  | { type: "GetSettlementWindows"; value: GetSettlementWindows }
  | { type: "GetSettlements"; value: GetSettlements }
  | { type: "CreateSettlement"; value: NewSettlement };