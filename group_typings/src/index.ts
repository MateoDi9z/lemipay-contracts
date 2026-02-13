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
    contractId: "CABYTW7GMOYRDOEYUTFQOFTYGPEFUZOOGYDIJLSYLDP7XFWQ4A2TFXP2",
  }
} as const


export interface Group {
  approvals_required: u32;
  members: Array<string>;
}

export type DataKey = {tag: "Group", values: readonly [u64]} | {tag: "GroupCounter", values: void};

export interface Client {
  /**
   * Construct and simulate a get_group transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns group data
   */
  get_group: ({group_id}: {group_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Group>>

  /**
   * Construct and simulate a get_members transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns members of a group
   */
  get_members: ({group_id}: {group_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Array<string>>>

  /**
   * Construct and simulate a create_group transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Creates a new group and returns group_id
   */
  create_group: ({members, approvals_required}: {members: Array<string>, approvals_required: u32}, options?: MethodOptions) => Promise<AssembledTransaction<u64>>

  /**
   * Construct and simulate a get_approval_rule transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns approval rule
   */
  get_approval_rule: ({group_id}: {group_id: u64}, options?: MethodOptions) => Promise<AssembledTransaction<u32>>

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
      new ContractSpec([ "AAAAAQAAAAAAAAAAAAAABUdyb3VwAAAAAAAAAgAAAAAAAAASYXBwcm92YWxzX3JlcXVpcmVkAAAAAAAEAAAAAAAAAAdtZW1iZXJzAAAAA+oAAAAT",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAAAgAAAAEAAAAAAAAABUdyb3VwAAAAAAAAAQAAAAYAAAAAAAAAAAAAAAxHcm91cENvdW50ZXI=",
        "AAAAAAAAABJSZXR1cm5zIGdyb3VwIGRhdGEAAAAAAAlnZXRfZ3JvdXAAAAAAAAABAAAAAAAAAAhncm91cF9pZAAAAAYAAAABAAAH0AAAAAVHcm91cAAAAA==",
        "AAAAAAAAABpSZXR1cm5zIG1lbWJlcnMgb2YgYSBncm91cAAAAAAAC2dldF9tZW1iZXJzAAAAAAEAAAAAAAAACGdyb3VwX2lkAAAABgAAAAEAAAPqAAAAEw==",
        "AAAAAAAAAChDcmVhdGVzIGEgbmV3IGdyb3VwIGFuZCByZXR1cm5zIGdyb3VwX2lkAAAADGNyZWF0ZV9ncm91cAAAAAIAAAAAAAAAB21lbWJlcnMAAAAD6gAAABMAAAAAAAAAEmFwcHJvdmFsc19yZXF1aXJlZAAAAAAABAAAAAEAAAAG",
        "AAAAAAAAABVSZXR1cm5zIGFwcHJvdmFsIHJ1bGUAAAAAAAARZ2V0X2FwcHJvdmFsX3J1bGUAAAAAAAABAAAAAAAAAAhncm91cF9pZAAAAAYAAAABAAAABA==" ]),
      options
    )
  }
  public readonly fromJSON = {
    get_group: this.txFromJSON<Group>,
        get_members: this.txFromJSON<Array<string>>,
        create_group: this.txFromJSON<u64>,
        get_approval_rule: this.txFromJSON<u32>
  }
}