export enum DiagnosticStatus {
    Ok = 0,
    Warn = 1,
    Error = 2,
    Stale = 3,
}

export interface DiagnosticKeyValue {
    key: string;
    value: string;
}


export interface OdometryDelta {
    OdometryDelta: {
        start_time: bigint;
        end_time: bigint;
        delta_position: [number, number];
        delta_orientation: number;
    }
}

export interface DiagnosticMsg {
    DiagnosticMsg: {
        level: DiagnosticStatus;
        name: string;
        message: string;
        values: DiagnosticKeyValue[];
    }
}
export interface SubscriptionRequest {
    SubscriptionRequest: {
        topics: string[];
    }
}

export interface UnknownPacket { [key: string]: unknown }

export interface PacketFormat<T> {
    to: number | null;
    from: number | null;
    time: bigint;
    id: number;
    data: T;
}

export type AnyPacketData = OdometryDelta | DiagnosticMsg | SubscriptionRequest | UnknownPacket
export type AnyPacketFormat = PacketFormat<AnyPacketData>;