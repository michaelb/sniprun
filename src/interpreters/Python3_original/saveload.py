# function has a complicated name so it NEVER gets overriden by user's defs
def sniprun142859_save(filename='memo', globals_=None):

    globals_ = globals_ or globals()
    import klepto
    dict_to_save = {}
    for key, value in globals_.items():
        if not key.startswith('__'):
            dict_to_save[key] = value
            # print('added %s to save' % key)
    a = klepto.archives.dir_archive(name=filename, dict=dict_to_save)
    a.dump()


def sniprun142859_load(filename='memo', globals_=None):
    import klepto
    b = klepto.archives.dir_archive(filename)
    for key in b.archive.keys():
        b.load(key)
        # print("loading %s " % key)
        globals()[key] = b[key]
