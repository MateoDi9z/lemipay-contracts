import { Buffer } from "buffer";
import { Address } from "@stellar/stellar-sdk";
import {
  AssembledTransaction,
  Client as ContractClient,
  ClientOptions as ContractClientOptions,
  MethodOptions,
  Result,
  Spec as ContractSpec,
} from "@stellar/stellar-sdk/contract";
import type {
  u32,
  i32,
  u64,
  i64,
  u128,
  i128,
  u256,
  i256,
  Option,
  Timepoint,
  Duration,
} from "@stellar/stellar-sdk/contract";
export * from "@stellar/stellar-sdk";
export * as contract from "@stellar/stellar-sdk/contract";
export * as rpc from "@stellar/stellar-sdk/rpc";

if (typeof window !== "undefined") {
  //@ts-ignore Buffer exists
  window.Buffer = window.Buffer || Buffer;
}


export const networks = {
  testnet: {
    networkPassphrase: "Test SDF Network ; September 2015",
    contractId: "CCCRRA4DSWP6UAJTF5XNK7VLD3TASQA3D274WBN5F3RDXLNI4DHJM7IZ",
  }
} as const


export interface FundRound {
  amount_of_members: u32;
  completed: boolean;
  funded_amount: i128;
  group_id: u64;
  total_amount: i128;
}


export interface ReleaseProposal {
  amount: i128;
  approvals: u32;
  destination: string;
  executed: boolean;
  group_id: u64;
}

export interface Client {
  /**
   * Construct and simulate a get_fund_round transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Get specific fund round details
   */
  get_fund_round: ({round_id}: {round_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<FundRound>>

  /**
   * Construct and simulate a approve_release transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * APPROVE RELEASE
   */
  approve_release: ({release_proposal_id, user}: {release_proposal_id: u64, user: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a create_treasury transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * ------------------------------------------------
   * CORE FUNCTIONS
   * ------------------------------------------------
   * CREATE TREASURY
   */
  create_treasury: ({group_id, user}: {group_id: u64, user: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a execute_release transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  execute_release: ({release_proposal_id}: {release_proposal_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a propose_release transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * PROPOSE RELEASE
   */
  propose_release: ({destination, amount, group_id, user}: {destination: string, amount: i128, group_id: u64, user: string}, options?: MethodOptions) => Promise<AssembledTransaction<u64>>

  /**
   * Construct and simulate a get_group_rounds transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * -------------------------------------------------
   * GETTERS
   * -------------------------------------------------
   * Get fund rounds of group
   */
  get_group_rounds: ({group_id}: {group_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Array<u64>>>

  /**
   * Construct and simulate a check_treasury_id transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * CHECK IF TREASURY EXISTS FOR GROUP ID
   */
  check_treasury_id: ({group_id}: {group_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a propose_fund_round transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  propose_fund_round: ({group_id, total_amount, user}: {group_id: u64, total_amount: i128, user: string}, options?: MethodOptions) => Promise<AssembledTransaction<u64>>

  /**
   * Construct and simulate a get_release_proposal transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Get specific release proposal details
   */
  get_release_proposal: ({proposal_id}: {proposal_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<ReleaseProposal>>

  /**
   * Construct and simulate a get_user_contribution transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Get user's contribution to a fund round
   */
  get_user_contribution: ({round_id, user}: {round_id: u64, user: string}, options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a contribute_to_fund_round transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  contribute_to_fund_round: ({round_id, amount, user}: {round_id: u64, amount: i128, user: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_release_proposals_of_group transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Get release proposal details by Group ID
   */
  get_release_proposals_of_group: ({group_id}: {group_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Array<u64>>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
    /** Options for initializing a Client as well as for calling a method, with extras specific to deploying. */
    options: MethodOptions &
      Omit<ContractClientOptions, "contractId"> & {
        /** The hash of the Wasm blob, which must already be installed on-chain. */
        wasmHash: Buffer | string;
        /** Salt used to generate the contract's ID. Passed through to {@link Operation.createCustomContract}. Default: random. */
        salt?: Buffer | Uint8Array;
        /** The format used to decode `wasmHash`, if it's provided as a string. */
        format?: "hex" | "base64";
      }
  ): Promise<AssembledTransaction<T>> {
    return ContractClient.deploy(null, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAQAAAAAAAAAAAAAACUZ1bmRSb3VuZAAAAAAAAAUAAAAAAAAAEWFtb3VudF9vZl9tZW1iZXJzAAAAAAAABAAAAAAAAAAJY29tcGxldGVkAAAAAAAAAQAAAAAAAAANZnVuZGVkX2Ftb3VudAAAAAAAAAsAAAAAAAAACGdyb3VwX2lkAAAABgAAAAAAAAAMdG90YWxfYW1vdW50AAAACw==",
        "AAAAAQAAAAAAAAAAAAAAD1JlbGVhc2VQcm9wb3NhbAAAAAAFAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAACWFwcHJvdmFscwAAAAAAAAQAAAAAAAAAC2Rlc3RpbmF0aW9uAAAAABMAAAAAAAAACGV4ZWN1dGVkAAAAAQAAAAAAAAAIZ3JvdXBfaWQAAAAG",
        "AAAAAAAAAB9HZXQgc3BlY2lmaWMgZnVuZCByb3VuZCBkZXRhaWxzAAAAAA5nZXRfZnVuZF9yb3VuZAAAAAAAAQAAAAAAAAAIcm91bmRfaWQAAAAGAAAAAQAAB9AAAAAJRnVuZFJvdW5kAAAA",
        "AAAAAAAAAA9BUFBST1ZFIFJFTEVBU0UAAAAAD2FwcHJvdmVfcmVsZWFzZQAAAAACAAAAAAAAABNyZWxlYXNlX3Byb3Bvc2FsX2lkAAAAAAYAAAAAAAAABHVzZXIAAAATAAAAAA==",
        "AAAAAAAAAIAtLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0KQ09SRSBGVU5DVElPTlMKLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tCkNSRUFURSBUUkVBU1VSWQAAAA9jcmVhdGVfdHJlYXN1cnkAAAAAAgAAAAAAAAAIZ3JvdXBfaWQAAAAGAAAAAAAAAAR1c2VyAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAAPZXhlY3V0ZV9yZWxlYXNlAAAAAAEAAAAAAAAAE3JlbGVhc2VfcHJvcG9zYWxfaWQAAAAABgAAAAA=",
        "AAAAAAAAAA9QUk9QT1NFIFJFTEVBU0UAAAAAD3Byb3Bvc2VfcmVsZWFzZQAAAAAEAAAAAAAAAAtkZXN0aW5hdGlvbgAAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAACGdyb3VwX2lkAAAABgAAAAAAAAAEdXNlcgAAABMAAAABAAAABg==",
        "AAAAAAAAAIQtLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tCkdFVFRFUlMKLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLS0tLQpHZXQgZnVuZCByb3VuZHMgb2YgZ3JvdXAAAAAQZ2V0X2dyb3VwX3JvdW5kcwAAAAEAAAAAAAAACGdyb3VwX2lkAAAABgAAAAEAAAPqAAAABg==",
        "AAAAAAAAACVDSEVDSyBJRiBUUkVBU1VSWSBFWElTVFMgRk9SIEdST1VQIElEAAAAAAAAEWNoZWNrX3RyZWFzdXJ5X2lkAAAAAAAAAQAAAAAAAAAIZ3JvdXBfaWQAAAAGAAAAAQAAAAE=",
        "AAAAAAAAAAAAAAAScHJvcG9zZV9mdW5kX3JvdW5kAAAAAAADAAAAAAAAAAhncm91cF9pZAAAAAYAAAAAAAAADHRvdGFsX2Ftb3VudAAAAAsAAAAAAAAABHVzZXIAAAATAAAAAQAAAAY=",
        "AAAAAAAAACVHZXQgc3BlY2lmaWMgcmVsZWFzZSBwcm9wb3NhbCBkZXRhaWxzAAAAAAAAFGdldF9yZWxlYXNlX3Byb3Bvc2FsAAAAAQAAAAAAAAALcHJvcG9zYWxfaWQAAAAABgAAAAEAAAfQAAAAD1JlbGVhc2VQcm9wb3NhbAA=",
        "AAAAAAAAACdHZXQgdXNlcidzIGNvbnRyaWJ1dGlvbiB0byBhIGZ1bmQgcm91bmQAAAAAFWdldF91c2VyX2NvbnRyaWJ1dGlvbgAAAAAAAAIAAAAAAAAACHJvdW5kX2lkAAAABgAAAAAAAAAEdXNlcgAAABMAAAABAAAACw==",
        "AAAAAAAAAAAAAAAYY29udHJpYnV0ZV90b19mdW5kX3JvdW5kAAAAAwAAAAAAAAAIcm91bmRfaWQAAAAGAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAABHVzZXIAAAATAAAAAA==",
        "AAAAAAAAAChHZXQgcmVsZWFzZSBwcm9wb3NhbCBkZXRhaWxzIGJ5IEdyb3VwIElEAAAAHmdldF9yZWxlYXNlX3Byb3Bvc2Fsc19vZl9ncm91cAAAAAAAAQAAAAAAAAAIZ3JvdXBfaWQAAAAGAAAAAQAAA+oAAAAG" ]),
      options
    )
  }
  public readonly fromJSON = {
    get_fund_round: this.txFromJSON<FundRound>,
        approve_release: this.txFromJSON<null>,
        create_treasury: this.txFromJSON<null>,
        execute_release: this.txFromJSON<null>,
        propose_release: this.txFromJSON<u64>,
        get_group_rounds: this.txFromJSON<Array<u64>>,
        check_treasury_id: this.txFromJSON<boolean>,
        propose_fund_round: this.txFromJSON<u64>,
        get_release_proposal: this.txFromJSON<ReleaseProposal>,
        get_user_contribution: this.txFromJSON<i128>,
        contribute_to_fund_round: this.txFromJSON<null>,
        get_release_proposals_of_group: this.txFromJSON<Array<u64>>
  }
}