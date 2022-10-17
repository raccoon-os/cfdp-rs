use std::{
    collections::HashMap,
    net::UdpSocket,
    sync::{atomic::AtomicBool, Arc},
    thread,
    time::Duration,
};

use camino::Utf8PathBuf;
use cfdp_core::{
    daemon::PutRequest,
    filestore::{FileStore, NativeFileStore},
    pdu::{Condition, EntityID, PDUDirective, TransmissionMode},
    transport::{PDUTransport, UdpTransport},
    user::User,
};
use rstest::{fixture, rstest};
use tempfile::TempDir;

mod common;
use common::{
    create_daemons, get_filestore, tempdir_fixture, terminate, EntityConstructorReturn,
    LossyTransport, TransportIssue,
};

#[fixture]
#[once]
fn fixture_f2s1(
    tempdir_fixture: &TempDir,
    get_filestore: &(&'static String, Arc<NativeFileStore>),
    terminate: &Arc<AtomicBool>,
) -> EntityConstructorReturn {
    let (_, filestore) = get_filestore;

    let remote_udp = UdpSocket::bind("127.0.0.1:0").expect("Unable to bind remote UDP.");
    let remote_addr = remote_udp.local_addr().expect("Cannot find local address.");

    let local_udp = UdpSocket::bind("127.0.0.1:0").expect("Unable to bind local UDP.");
    let local_addr = local_udp.local_addr().expect("Cannot find local address.");

    let entity_map = {
        let mut temp = HashMap::new();
        temp.insert(EntityID::from(0_u16), local_addr);
        temp.insert(EntityID::from(1_u16), remote_addr);
        temp
    };

    let local_transport = LossyTransport::try_from((
        local_udp,
        entity_map.clone(),
        TransportIssue::Once(PDUDirective::Metadata),
    ))
    .expect("Unable to make Lossy Transport.");
    let remote_transport =
        UdpTransport::try_from((remote_udp, entity_map)).expect("Unable to make UdpTransport.");

    let remote_transport_map: HashMap<Vec<EntityID>, Box<dyn PDUTransport + Send>> =
        HashMap::from([(
            vec![EntityID::from(0_u16)],
            Box::new(remote_transport) as Box<dyn PDUTransport + Send>,
        )]);

    let local_transport_map: HashMap<Vec<EntityID>, Box<dyn PDUTransport + Send>> =
        HashMap::from([(
            vec![EntityID::from(1_u16)],
            Box::new(local_transport) as Box<dyn PDUTransport + Send>,
        )]);

    let path = Utf8PathBuf::from(
        tempdir_fixture
            .path()
            .as_os_str()
            .to_str()
            .expect("Unable to coerce tmp path to String."),
    );
    let (path, local, remote) = create_daemons(
        path.as_path(),
        filestore.clone(),
        local_transport_map,
        remote_transport_map,
        "f2s1_local.socket",
        "f2s1_remote.socket",
        terminate.clone(),
    );
    (path, filestore.clone(), local, remote)
}

#[rstest]
#[cfg_attr(target_os = "windows", ignore)]
#[timeout(Duration::from_secs(5))]
// Series F2
// Sequence 1 Test
// Test goal:
//  - Recover from Loss of Metadata PDU
// Configuration:
//  - Acknowledged
//  - File Size: Medium
//  - Drop first instance of Metadata PDU
fn f2s1(fixture_f2s1: &'static EntityConstructorReturn) {
    // let mut user = User::new(Some(_local_path))
    let (local_path, filestore, _local, _remote) = fixture_f2s1;
    let mut user = User::new(Some(local_path)).expect("User Cannot connect to Daemon.");

    let out_file: Utf8PathBuf = "remote/medium_f2s1.txt".into();
    let path_to_out = filestore.get_native_path(&out_file);

    user.put(PutRequest {
        source_filename: "local/medium.txt".into(),
        destination_filename: out_file,
        destination_entity_id: EntityID::from(1_u16),
        transmission_mode: TransmissionMode::Acknowledged,
        filestore_requests: vec![],
        message_to_user: vec![],
    })
    .expect("unable to send put request.");

    while !path_to_out.exists() {
        thread::sleep(Duration::from_millis(1))
    }

    assert!(path_to_out.exists());
}

#[fixture]
#[once]
fn fixture_f2s2(
    tempdir_fixture: &TempDir,
    get_filestore: &(&'static String, Arc<NativeFileStore>),
    terminate: &Arc<AtomicBool>,
) -> EntityConstructorReturn {
    let (_, filestore) = get_filestore;

    let remote_udp = UdpSocket::bind("127.0.0.1:0").expect("Unable to bind remote UDP.");
    let remote_addr = remote_udp.local_addr().expect("Cannot find local address.");

    let local_udp = UdpSocket::bind("127.0.0.1:0").expect("Unable to bind local UDP.");
    let local_addr = local_udp.local_addr().expect("Cannot find local address.");

    let entity_map = {
        let mut temp = HashMap::new();
        temp.insert(EntityID::from(0_u16), local_addr);
        temp.insert(EntityID::from(1_u16), remote_addr);
        temp
    };

    let local_transport = LossyTransport::try_from((
        local_udp,
        entity_map.clone(),
        TransportIssue::Once(PDUDirective::EoF),
    ))
    .expect("Unable to make Lossy Transport.");
    let remote_transport =
        UdpTransport::try_from((remote_udp, entity_map)).expect("Unable to make UdpTransport.");

    let remote_transport_map: HashMap<Vec<EntityID>, Box<dyn PDUTransport + Send>> =
        HashMap::from([(
            vec![EntityID::from(0_u16)],
            Box::new(remote_transport) as Box<dyn PDUTransport + Send>,
        )]);

    let local_transport_map: HashMap<Vec<EntityID>, Box<dyn PDUTransport + Send>> =
        HashMap::from([(
            vec![EntityID::from(1_u16)],
            Box::new(local_transport) as Box<dyn PDUTransport + Send>,
        )]);

    let path = Utf8PathBuf::from(
        tempdir_fixture
            .path()
            .as_os_str()
            .to_str()
            .expect("Unable to coerce tmp path to String."),
    );
    let (path, local, remote) = create_daemons(
        path.as_path(),
        filestore.clone(),
        local_transport_map,
        remote_transport_map,
        "f2s2_local.socket",
        "f2s2_remote.socket",
        terminate.clone(),
    );
    (path, filestore.clone(), local, remote)
}

#[rstest]
#[cfg_attr(target_os = "windows", ignore)]
#[timeout(Duration::from_secs(5))]
// Series F2
// Sequence 2 Test
// Test goal:
//  - Recover from Loss of EoF PDU
// Configuration:
//  - Acknowledged
//  - File Size: Medium
//  - Drop first instance of EoF PDU
fn f2s2(fixture_f2s2: &'static EntityConstructorReturn) {
    // let mut user = User::new(Some(_local_path))
    let (local_path, filestore, _local, _remote) = fixture_f2s2;
    let mut user = User::new(Some(local_path)).expect("User Cannot connect to Daemon.");

    let out_file: Utf8PathBuf = "remote/medium_f2s2.txt".into();
    let path_to_out = filestore.get_native_path(&out_file);

    user.put(PutRequest {
        source_filename: "local/medium.txt".into(),
        destination_filename: out_file,
        destination_entity_id: EntityID::from(1_u16),
        transmission_mode: TransmissionMode::Acknowledged,
        filestore_requests: vec![],
        message_to_user: vec![],
    })
    .expect("unable to send put request.");

    while !path_to_out.exists() {
        thread::sleep(Duration::from_millis(1))
    }

    assert!(path_to_out.exists());
}

#[fixture]
#[once]
fn fixture_f2s3(
    tempdir_fixture: &TempDir,
    get_filestore: &(&'static String, Arc<NativeFileStore>),
    terminate: &Arc<AtomicBool>,
) -> EntityConstructorReturn {
    let (_, filestore) = get_filestore;
    let remote_udp = UdpSocket::bind("127.0.0.1:0").expect("Unable to bind remote UDP.");
    let remote_addr = remote_udp.local_addr().expect("Cannot find local address.");

    let local_udp = UdpSocket::bind("127.0.0.1:0").expect("Unable to bind local UDP.");
    let local_addr = local_udp.local_addr().expect("Cannot find local address.");

    let entity_map = {
        let mut temp = HashMap::new();
        temp.insert(EntityID::from(0_u16), local_addr);
        temp.insert(EntityID::from(1_u16), remote_addr);
        temp
    };

    let local_transport = UdpTransport::try_from((local_udp, entity_map.clone()))
        .expect("Unable to make UdpTransport.");
    let remote_transport = LossyTransport::try_from((
        remote_udp,
        entity_map,
        TransportIssue::Once(PDUDirective::Finished),
    ))
    .expect("Unable to make Lossy Transport.");

    let remote_transport_map: HashMap<Vec<EntityID>, Box<dyn PDUTransport + Send>> =
        HashMap::from([(
            vec![EntityID::from(0_u16)],
            Box::new(remote_transport) as Box<dyn PDUTransport + Send>,
        )]);

    let local_transport_map: HashMap<Vec<EntityID>, Box<dyn PDUTransport + Send>> =
        HashMap::from([(
            vec![EntityID::from(1_u16)],
            Box::new(local_transport) as Box<dyn PDUTransport + Send>,
        )]);

    let path = Utf8PathBuf::from(
        tempdir_fixture
            .path()
            .as_os_str()
            .to_str()
            .expect("Unable to coerce tmp path to String."),
    );
    let (path, local, remote) = create_daemons(
        path.as_path(),
        filestore.clone(),
        local_transport_map,
        remote_transport_map,
        "f2s3_local.socket",
        "f2s3_remote.socket",
        terminate.clone(),
    );
    (path, filestore.clone(), local, remote)
}

#[rstest]
#[cfg_attr(target_os = "windows", ignore)]
#[timeout(Duration::from_secs(5))]
// Series F2
// Sequence 3 Test
// Test goal:
//  - Recover from Loss of Finished PDU
// Configuration:
//  - Acknowledged
//  - File Size: Medium
//  - Drop first instance of Finished PDU
fn f2s3(fixture_f2s3: &'static EntityConstructorReturn) {
    // let mut user = User::new(Some(_local_path))
    let (local_path, filestore, _local, _remote) = fixture_f2s3;
    let mut user = User::new(Some(local_path)).expect("User Cannot connect to Daemon.");

    let out_file: Utf8PathBuf = "remote/medium_f2s3.txt".into();
    let path_to_out = filestore.get_native_path(&out_file);

    user.put(PutRequest {
        source_filename: "local/medium.txt".into(),
        destination_filename: out_file,
        destination_entity_id: EntityID::from(1_u16),
        transmission_mode: TransmissionMode::Acknowledged,
        filestore_requests: vec![],
        message_to_user: vec![],
    })
    .expect("unable to send put request.");

    while !path_to_out.exists() {
        thread::sleep(Duration::from_millis(1))
    }

    assert!(path_to_out.exists());
}

#[fixture]
#[once]
fn fixture_f2s4(
    tempdir_fixture: &TempDir,
    get_filestore: &(&'static String, Arc<NativeFileStore>),
    terminate: &Arc<AtomicBool>,
) -> EntityConstructorReturn {
    let (_, filestore) = get_filestore;
    let remote_udp = UdpSocket::bind("127.0.0.1:0").expect("Unable to bind remote UDP.");
    let remote_addr = remote_udp.local_addr().expect("Cannot find local address.");

    let local_udp = UdpSocket::bind("127.0.0.1:0").expect("Unable to bind local UDP.");
    let local_addr = local_udp.local_addr().expect("Cannot find local address.");

    let entity_map = {
        let mut temp = HashMap::new();
        temp.insert(EntityID::from(0_u16), local_addr);
        temp.insert(EntityID::from(1_u16), remote_addr);
        temp
    };

    let local_transport = UdpTransport::try_from((local_udp, entity_map.clone()))
        .expect("Unable to make UdpTransport.");
    let remote_transport = LossyTransport::try_from((
        remote_udp,
        entity_map,
        TransportIssue::Once(PDUDirective::Ack),
    ))
    .expect("Unable to make Lossy Transport.");

    let remote_transport_map: HashMap<Vec<EntityID>, Box<dyn PDUTransport + Send>> =
        HashMap::from([(
            vec![EntityID::from(0_u16)],
            Box::new(remote_transport) as Box<dyn PDUTransport + Send>,
        )]);

    let local_transport_map: HashMap<Vec<EntityID>, Box<dyn PDUTransport + Send>> =
        HashMap::from([(
            vec![EntityID::from(1_u16)],
            Box::new(local_transport) as Box<dyn PDUTransport + Send>,
        )]);

    let path = Utf8PathBuf::from(
        tempdir_fixture
            .path()
            .as_os_str()
            .to_str()
            .expect("Unable to coerce tmp path to String."),
    );
    let (path, local, remote) = create_daemons(
        path.as_path(),
        filestore.clone(),
        local_transport_map,
        remote_transport_map,
        "f2s4_local.socket",
        "f2s4_remote.socket",
        terminate.clone(),
    );
    (path, filestore.clone(), local, remote)
}

#[rstest]
#[cfg_attr(target_os = "windows", ignore)]
#[timeout(Duration::from_secs(5))]
// Series F2
// Sequence 3 Test
// Test goal:
//  - Recover from Loss of ACK(EOF) PDU
// Configuration:
//  - Acknowledged
//  - File Size: Medium
//  - Drop first instance of ACK(EOF) PDU
fn f2s4(fixture_f2s4: &'static EntityConstructorReturn) {
    // let mut user = User::new(Some(_local_path))
    let (local_path, filestore, _local, _remote) = fixture_f2s4;
    let mut user = User::new(Some(local_path)).expect("User Cannot connect to Daemon.");

    let out_file: Utf8PathBuf = "remote/medium_f2s4.txt".into();
    let path_to_out = filestore.get_native_path(&out_file);

    user.put(PutRequest {
        source_filename: "local/medium.txt".into(),
        destination_filename: out_file,
        destination_entity_id: EntityID::from(1_u16),
        transmission_mode: TransmissionMode::Acknowledged,
        filestore_requests: vec![],
        message_to_user: vec![],
    })
    .expect("unable to send put request.");

    while !path_to_out.exists() {
        thread::sleep(Duration::from_millis(1))
    }

    assert!(path_to_out.exists());
}

#[fixture]
#[once]
fn fixture_f2s5(
    tempdir_fixture: &TempDir,
    get_filestore: &(&'static String, Arc<NativeFileStore>),
    terminate: &Arc<AtomicBool>,
) -> EntityConstructorReturn {
    let (_, filestore) = get_filestore;
    let remote_udp = UdpSocket::bind("127.0.0.1:0").expect("Unable to bind remote UDP.");
    let remote_addr = remote_udp.local_addr().expect("Cannot find local address.");

    let local_udp = UdpSocket::bind("127.0.0.1:0").expect("Unable to bind local UDP.");
    let local_addr = local_udp.local_addr().expect("Cannot find local address.");

    let entity_map = {
        let mut temp = HashMap::new();
        temp.insert(EntityID::from(0_u16), local_addr);
        temp.insert(EntityID::from(1_u16), remote_addr);
        temp
    };

    let local_transport = LossyTransport::try_from((
        local_udp,
        entity_map.clone(),
        TransportIssue::Once(PDUDirective::Ack),
    ))
    .expect("Unable to make UdpTransport.");
    let remote_transport =
        UdpTransport::try_from((remote_udp, entity_map)).expect("Unable to make Lossy Transport.");

    let remote_transport_map: HashMap<Vec<EntityID>, Box<dyn PDUTransport + Send>> =
        HashMap::from([(
            vec![EntityID::from(0_u16)],
            Box::new(remote_transport) as Box<dyn PDUTransport + Send>,
        )]);

    let local_transport_map: HashMap<Vec<EntityID>, Box<dyn PDUTransport + Send>> =
        HashMap::from([(
            vec![EntityID::from(1_u16)],
            Box::new(local_transport) as Box<dyn PDUTransport + Send>,
        )]);

    let path = Utf8PathBuf::from(
        tempdir_fixture
            .path()
            .as_os_str()
            .to_str()
            .expect("Unable to coerce tmp path to String."),
    );
    let (path, local, remote) = create_daemons(
        path.as_path(),
        filestore.clone(),
        local_transport_map,
        remote_transport_map,
        "f2s5_local.socket",
        "f2s5_remote.socket",
        terminate.clone(),
    );
    (path, filestore.clone(), local, remote)
}

#[rstest]
#[cfg_attr(target_os = "windows", ignore)]
#[timeout(Duration::from_secs(5))]
// Series F2
// Sequence 5 Test
// Test goal:
//  - Recover from Loss of ACK(Fin) PDU
// Configuration:
//  - Acknowledged
//  - File Size: Medium
//  - Drop first instance of ACK(Fin) PDU
fn f2s5(fixture_f2s5: &'static EntityConstructorReturn) {
    // let mut user = User::new(Some(_local_path))
    let (local_path, filestore, _local, _remote) = fixture_f2s5;
    let mut user = User::new(Some(local_path)).expect("User Cannot connect to Daemon.");

    let out_file: Utf8PathBuf = "remote/medium_f2s5.txt".into();
    let path_to_out = filestore.get_native_path(&out_file);

    user.put(PutRequest {
        source_filename: "local/medium.txt".into(),
        destination_filename: out_file,
        destination_entity_id: EntityID::from(1_u16),
        transmission_mode: TransmissionMode::Acknowledged,
        filestore_requests: vec![],
        message_to_user: vec![],
    })
    .expect("unable to send put request.");

    while !path_to_out.exists() {
        thread::sleep(Duration::from_millis(1))
    }

    assert!(path_to_out.exists());
}

#[fixture]
#[once]
fn fixture_f2s6(
    tempdir_fixture: &TempDir,
    get_filestore: &(&'static String, Arc<NativeFileStore>),
    terminate: &Arc<AtomicBool>,
) -> EntityConstructorReturn {
    let (_, filestore) = get_filestore;
    let remote_udp = UdpSocket::bind("127.0.0.1:0").expect("Unable to bind remote UDP.");
    let remote_addr = remote_udp.local_addr().expect("Cannot find local address.");

    let local_udp = UdpSocket::bind("127.0.0.1:0").expect("Unable to bind local UDP.");
    let local_addr = local_udp.local_addr().expect("Cannot find local address.");

    let entity_map = {
        let mut temp = HashMap::new();
        temp.insert(EntityID::from(0_u16), local_addr);
        temp.insert(EntityID::from(1_u16), remote_addr);
        temp
    };

    let local_transport =
        LossyTransport::try_from((local_udp, entity_map.clone(), TransportIssue::Every))
            .expect("Unable to make UdpTransport.");
    let remote_transport =
        LossyTransport::try_from((remote_udp, entity_map, TransportIssue::Every))
            .expect("Unable to make Lossy Transport.");

    let remote_transport_map: HashMap<Vec<EntityID>, Box<dyn PDUTransport + Send>> =
        HashMap::from([(
            vec![EntityID::from(0_u16)],
            Box::new(remote_transport) as Box<dyn PDUTransport + Send>,
        )]);

    let local_transport_map: HashMap<Vec<EntityID>, Box<dyn PDUTransport + Send>> =
        HashMap::from([(
            vec![EntityID::from(1_u16)],
            Box::new(local_transport) as Box<dyn PDUTransport + Send>,
        )]);

    let path = Utf8PathBuf::from(
        tempdir_fixture
            .path()
            .as_os_str()
            .to_str()
            .expect("Unable to coerce tmp path to String."),
    );
    let (path, local, remote) = create_daemons(
        path.as_path(),
        filestore.clone(),
        local_transport_map,
        remote_transport_map,
        "f2s6_local.socket",
        "f2s6_remote.socket",
        terminate.clone(),
    );
    (path, filestore.clone(), local, remote)
}

#[rstest]
#[cfg_attr(target_os = "windows", ignore)]
#[timeout(Duration::from_secs(5))]
// Series F2
// Sequence 6 Test
// Test goal:
//  - Recover from noisy environment
// Configuration:
//  - Acknowledged
//  - File Size: Medium
//  - Drop first instance of Every non-EOF pdu in both directions
fn f2s6(fixture_f2s6: &'static EntityConstructorReturn) {
    // let mut user = User::new(Some(_local_path))
    let (local_path, filestore, _local, _remote) = fixture_f2s6;
    let mut user = User::new(Some(local_path)).expect("User Cannot connect to Daemon.");

    let out_file: Utf8PathBuf = "remote/medium_f2s6.txt".into();
    let path_to_out = filestore.get_native_path(&out_file);

    user.put(PutRequest {
        source_filename: "local/medium.txt".into(),
        destination_filename: out_file,
        destination_entity_id: EntityID::from(1_u16),
        transmission_mode: TransmissionMode::Acknowledged,
        filestore_requests: vec![],
        message_to_user: vec![],
    })
    .expect("unable to send put request.");

    while !path_to_out.exists() {
        thread::sleep(Duration::from_millis(1))
    }

    assert!(path_to_out.exists());
}

#[fixture]
#[once]
fn fixture_f2s7(
    tempdir_fixture: &TempDir,
    get_filestore: &(&'static String, Arc<NativeFileStore>),
    terminate: &Arc<AtomicBool>,
) -> EntityConstructorReturn {
    let (_, filestore) = get_filestore;
    let remote_udp = UdpSocket::bind("127.0.0.1:0").expect("Unable to bind remote UDP.");
    let remote_addr = remote_udp.local_addr().expect("Cannot find local address.");

    let local_udp = UdpSocket::bind("127.0.0.1:0").expect("Unable to bind local UDP.");
    let local_addr = local_udp.local_addr().expect("Cannot find local address.");

    let entity_map = {
        let mut temp = HashMap::new();
        temp.insert(EntityID::from(0_u16), local_addr);
        temp.insert(EntityID::from(1_u16), remote_addr);
        temp
    };

    let local_transport = UdpTransport::try_from((local_udp, entity_map.clone()))
        .expect("Unable to make UdpTransport.");
    let remote_transport = LossyTransport::try_from((
        remote_udp,
        entity_map,
        TransportIssue::All(vec![PDUDirective::Finished, PDUDirective::Ack]),
    ))
    .expect("Unable to make Lossy Transport.");

    let remote_transport_map: HashMap<Vec<EntityID>, Box<dyn PDUTransport + Send>> =
        HashMap::from([(
            vec![EntityID::from(0_u16)],
            Box::new(remote_transport) as Box<dyn PDUTransport + Send>,
        )]);

    let local_transport_map: HashMap<Vec<EntityID>, Box<dyn PDUTransport + Send>> =
        HashMap::from([(
            vec![EntityID::from(1_u16)],
            Box::new(local_transport) as Box<dyn PDUTransport + Send>,
        )]);

    let path = Utf8PathBuf::from(
        tempdir_fixture
            .path()
            .as_os_str()
            .to_str()
            .expect("Unable to coerce tmp path to String."),
    );
    let (path, local, remote) = create_daemons(
        path.as_path(),
        filestore.clone(),
        local_transport_map,
        remote_transport_map,
        "f2s7_local.socket",
        "f2s7_remote.socket",
        terminate.clone(),
    );
    (path, filestore.clone(), local, remote)
}

#[rstest]
#[cfg_attr(target_os = "windows", ignore)]
#[timeout(Duration::from_secs(10))]
// Series F2
// Sequence 7 Test
// Test goal:
//  - check ACK limit reached at Sender
// Configuration:
//  - Acknowledged
//  - File Size: Medium
//  - Drop all ACK and Finished PDUs
fn f2s7(fixture_f2s7: &'static EntityConstructorReturn) {
    // let mut user = User::new(Some(_local_path))
    let (local_path, filestore, _local, _remote) = fixture_f2s7;
    let mut user = User::new(Some(local_path)).expect("User Cannot connect to Daemon.");

    let out_file: Utf8PathBuf = "remote/medium_f2s7.txt".into();
    let path_to_out = filestore.get_native_path(&out_file);

    let id = user
        .put(PutRequest {
            source_filename: "local/medium.txt".into(),
            destination_filename: out_file,
            destination_entity_id: EntityID::from(1_u16),
            transmission_mode: TransmissionMode::Acknowledged,
            filestore_requests: vec![],
            message_to_user: vec![],
        })
        .expect("unable to send put request.");

    while !path_to_out.exists() {
        thread::sleep(Duration::from_millis(1))
    }
    assert!(path_to_out.exists());
    // wait long enough for the ack limit to be reached

    let mut report = user
        .report(id.clone())
        .expect("Unable to send Report Request.")
        .unwrap();

    while report.condition != Condition::PositiveLimitReached {
        thread::sleep(Duration::from_millis(1));
        report = user
            .report(id.clone())
            .expect("Unable to send Report Request.")
            .unwrap();
    }

    assert_eq!(report.condition, Condition::PositiveLimitReached)
}
