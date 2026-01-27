export const DiagnosticStatus = {
    Ok: 0,
    Warn: 1,
    Error: 2,
    Stale: 3,
} as const

export type DiagnosticStatus = typeof DiagnosticStatus[keyof typeof DiagnosticStatus]

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

export interface  PositionEstimate {
    PositionEstimate: {
        timestamp: bigint;
        position: [number, number];
        orientation: number;
    }
}


export interface MotionTargetRequest {
    MotionTargetRequest: {
        linear: [number, number];
        angular: number;
        motion_mode: 'Position' | 'Velocity' | 'Stop';
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

export type AnyPacketData = OdometryDelta | DiagnosticMsg | SubscriptionRequest | PositionEstimate | MotionTargetRequest | UnknownPacket
export type AnyPacketFormat = PacketFormat<AnyPacketData>;