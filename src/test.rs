use super::Trie;

#[test]
fn test_gen() {
    let mut trie = Trie::default();
    trie.insert(vec![0x12, 0x34, 0x56, 0x78], "12345678");
    trie.insert(vec![0x12, 0x34, 0xab, 0xcd], "1234abcd");
    trie.insert(vec![0x12, 0x34, 0xab, 0xef], "1234abef");
    trie.insert(vec![0x12, 0x31, 0x11, 0x11], "12311111");
    trie.insert(vec![0x12, 0x30, 0x11, 0x11], "12301111");
    trie.insert(vec![0x12, 0xff], "12ff");
    trie.insert(vec![0x12], "12");
    trie.insert(vec![0x12, 0x01], "1201");
    trie.insert(vec![0x12, 0x30, 0x42], "123042");
    trie.insert(vec![0x12, 0x30], "1230");
    trie.insert(vec![0x92, 0x35, 0x00], "923500");

    assert_eq!(trie.get(&vec![0x12, 0x30, 0x11, 0x11]), Some(&"12301111"));

    {
        let prefix = vec![0x12, 0x30];
        let mut it = trie.prefix_iter(&prefix);
        let found = it.next().unwrap();
        assert_eq!(*found.1, "12301111");
        let found = it.next().unwrap();
        assert_eq!(*found.1, "123042");
        assert!(it.next().is_none());
    }

    {
        let prefix = vec![0x12, 0x30];
        let mut it = trie.prefix_iter(&prefix).include_prefix();
        let found = it.next().unwrap();
        assert_eq!(*found.1, "1230");
    }

    {
        let prefix = vec![];
        let mut it = trie.prefix_iter(&prefix);
        let found = it.next().unwrap();
        assert_eq!(*found.1, "12");
    }

    {
        let prefix = vec![0xff];
        let mut it = trie.prefix_iter(&prefix);
        assert!(it.next().is_none());
    }

    assert_eq!(trie.remove(&vec![0xf2, 0x30, 0x42]), false);
    assert_eq!(trie.get(&vec![0x12, 0x30, 0x42]), Some(&"123042"));
    assert_eq!(trie.remove(&vec![0x12, 0x30, 0x42]), true);
    assert_eq!(trie.get(&vec![0x12, 0x30, 0x42]), None);
    assert_eq!(trie.remove(&vec![0x12, 0x30, 0x42]), false);
    assert_eq!(trie.get(&vec![0x12, 0x30, 0x42]), None);

    let mut trie2 = Trie::default();
    trie2.insert("x", "x");
    assert_eq!(trie2.get(&"x"), Some(&"x"));
    assert_eq!(trie2.get_mut(&"x"), Some(&mut "x"));
    assert_eq!(trie2.remove(&"x"), true);
    assert_eq!(trie2.get(&"x"), None);
    assert_eq!(trie2.get_mut(&"x"), None);

    let mut trie3 = Trie::default();
    trie3.insert("z", "z");
    assert!(!trie3.is_empty());
    trie3.remove(&"y");
    assert!(!trie3.is_empty());
    trie3.remove(&"z");
    assert!(trie3.is_empty());
    trie3.insert("x", "x");
    trie3.insert("y", "y");
    trie3.remove(&"y");
    trie3.remove(&"x");
    assert!(trie3.is_empty());
    trie3.insert("x", "x");
    trie3.insert("y", "y");
    trie3.insert("z", "z");
    trie3.remove(&"y");
    assert!(!trie3.is_empty());
    trie3.remove(&"x");
    assert!(!trie3.is_empty());
    trie3.remove(&"z");
    assert!(trie3.is_empty());
}
