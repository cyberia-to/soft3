//! One-shot baseline capture, run against the unmodified v1 engine before migration.
use cell_engine::{Config, Engine, Error, GraphPort, Progress};
use cell_model::{Builder, Content, Head, Particle, Source};
use cell_node::{Graph, LocalWard, Rune};
use std::{cell::RefCell, collections::BTreeMap, io::Write, path::PathBuf};

struct WriteRecord {
    namespace: Particle,
    request: Particle,
    expected: Option<Head>,
    head: Head,
    ids: Vec<Particle>,
    claim: Option<(Particle, Particle)>,
}
struct Capture {
    graph: Graph,
    content: RefCell<BTreeMap<Particle, Content>>,
    writes: RefCell<Vec<WriteRecord>>,
    interrupt: std::cell::Cell<bool>,
}
impl Source for Capture {
    fn get(&self, id: &Particle) -> Result<Content, cell_model::Error> {
        self.graph.get(id)
    }
}
impl GraphPort for Capture {
    fn head(&self, namespace: Particle) -> Result<Option<Head>, Error> {
        self.graph.head(namespace)
    }
    fn resolve(&self, namespace: Particle, request: Particle) -> Result<Option<Head>, Error> {
        self.graph.resolve(namespace, request)
    }
    fn history(&self, namespace: Particle, after: Option<u64>, limit: usize) -> Result<Vec<Head>, Error> {
        self.graph.history(namespace, after, limit)
    }
    fn commit(&self, namespace: Particle, request: Particle, expected: Option<Head>, head: Head,
        content: Builder, claim: Option<(Particle, Particle)>) -> Result<Head, Error> {
        let ids = content.content.keys().copied().collect();
        self.content.borrow_mut().extend(content.content.iter().map(|(k,v)| (*k,v.clone())));
        let result = self.graph.commit(namespace, request, expected, head, content, claim)?;
        self.writes.borrow_mut().push(WriteRecord { namespace, request, expected, head, ids, claim });
        if self.interrupt.replace(false) {
            return Err(Error::CommitUnknown("baseline simulated lost reservation receipt".into()));
        }
        Ok(result)
    }
}
fn value(s: &str) -> Vec<u8> { cell_rune::value(s).unwrap() }
fn drive(e: &Engine<Capture, Rune>, id: Particle) -> Result<Progress, Error> {
    for _ in 0..100 {
        match e.tick(id)? { Progress::Advanced(_) => {}, other => return Ok(other) }
    }
    Err(Error::Budget)
}
fn uint(w: &mut impl Write, n: u64) { w.write_all(&n.to_le_bytes()).unwrap(); }
fn head(w: &mut impl Write, h: Head) { uint(w,h.index); w.write_all(&h.commit).unwrap(); }
fn hex(id: Particle) -> String { id.iter().map(|b| format!("{b:02x}")).collect() }
fn main() {
    let output = PathBuf::from(std::env::args_os().nth(1).expect("output directory"));
    std::fs::create_dir_all(&output).unwrap();
    let graph = Capture { graph: Graph::open(output.join("legacy-bbg")).unwrap(),
        content: RefCell::new(BTreeMap::new()), writes: RefCell::new(Vec::new()),
        interrupt: std::cell::Cell::new(false) };
    let e = Engine::with_ward(graph, Rune, LocalWard);
    let counter = e.create(b"~mem + event".to_vec(), value("0"), [1;32], Config::default()).unwrap();
    let admitted = e.submit(counter, [2;32], value("7"), None).unwrap();
    assert!(matches!(drive(&e,counter).unwrap(),Progress::Complete(_)));
    assert_eq!(e.inspect(counter).unwrap().state,value("7"));
    assert_eq!(e.submit(counter,[2;32],value("7"),None).unwrap(),admitted);
    let tool = e.create(b"host(event)".to_vec(), value("0"), [3;32],
        Config { allowed_acts: vec![0xAC75_0000_0000_0006], ..Config::default() }).unwrap();
    e.submit(tool,[4;32],value("21"),None).unwrap();
    let Progress::Awaiting {operation,..} = drive(&e,tool).unwrap() else { panic!("tool boundary") };
    let attempt = e.begin_attempt(tool,operation).unwrap();
    assert!(matches!(e.tick(tool).unwrap(),Progress::Unknown{..}));
    let reserved = e.create(b"~mem + event".to_vec(), value("3"), [5;32],Config::default()).unwrap();
    e.submit(reserved,[6;32],value("9"),None).unwrap();
    e.graph.interrupt.set(true);
    assert!(matches!(e.tick(reserved),Err(Error::CommitUnknown(_))));
    assert_eq!(e.inspect(reserved).unwrap().live.unwrap().reserved,1000);
    let mut f=std::fs::File::create(output.join("legacy-v1.capture")).unwrap();
    f.write_all(b"CELLCAP1").unwrap();
    let content=e.graph.content.borrow(); uint(&mut f,content.len() as u64);
    for (id,c) in content.iter() {
        f.write_all(id).unwrap(); f.write_all(&[u8::from(c.blob)]).unwrap();
        uint(&mut f,c.bytes.len() as u64); f.write_all(&c.bytes).unwrap();
    }
    let writes=e.graph.writes.borrow(); uint(&mut f,writes.len() as u64);
    for w in writes.iter() {
        f.write_all(&w.namespace).unwrap(); f.write_all(&w.request).unwrap();
        f.write_all(&[u8::from(w.expected.is_some())]).unwrap();
        if let Some(h)=w.expected {head(&mut f,h);}
        head(&mut f,w.head);uint(&mut f,w.ids.len() as u64);
        for id in &w.ids {f.write_all(id).unwrap();}
        f.write_all(&[u8::from(w.claim.is_some())]).unwrap();
        if let Some((k,v))=w.claim {f.write_all(&k).unwrap();f.write_all(&v).unwrap();}
    }
    f.sync_all().unwrap();
    let manifest=format!("counter={}\ncounter_state=7\ncompleted_request={}\ntool={}\noperation={}\nattempt={}\nreserved={}\nreserved_steps=1000\ncontents={}\nwrites={}\n",
        hex(counter),hex(admitted.commit),hex(tool),hex(operation),hex(attempt.attempt),hex(reserved),content.len(),writes.len());
    std::fs::write(output.join("legacy-v1.txt"),manifest.as_bytes()).unwrap();
    print!("{manifest}");
}
