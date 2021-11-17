import os
import urllib.request
import hashlib
import shutil
import sqlite3
#本项目使用到的所有函数
def add(path):
    file_list=[]
    for root,dir,files in os.walk(path):
        for file in files:
            file_list.append(file)
    file_list.sort()
    return file_list
def abspath(file_list,file_abspath):
    for i in file_list:
        file_abspath[i]=i.split(sep='/')[-2]
    return file_abspath

def cmp(newdeb,nowdeb):
    return(os.popen('dpkg --compare-versions '+newdeb+' gt '+nowdeb +'\n'+'echo $?').read().split('\n')[0])
def get_filedict(dir,fileDict):
    newDir=dir
    if os.path.isfile(dir):
        fileDict[os.path.basename(dir)]=dir
    elif os.path.isdir(dir):
        for s in os.listdir(dir):
            newDir=os.path.join(dir,s)
            get_filedict(newDir,fileDict)
    return fileDict

def get_oldpath(list_file):
    url='http://packages.aosc.io/cleanmirror/amd64/stable'
    headers={'User-Agent':'Mozilla/5.0 (Wi4ndows NT 6.1; WOW64; rv:23.0) Gecko/20100101 Firefox/23.0'}
    req=urllib.request.Request(url,headers=headers)
    page=urllib.request.urlopen(req)
    response=page.readlines()
    list_name=[]
    for i in response:
        i=str(i)
        list_name.append(i.split(sep='\\')[0])
    for i in list_name:
        list_file.append(i.split(sep='\'')[1])
    return list_file
def get_real_size(p_doc):
    size=0.0
    for root,dirs,files in os.walk(p_doc):
        size+=sum([os.path.getsize(os.path.join(root,file)) for file in files])
    #size=round(size/1024/1024/1024,2)
    size=round(size/1000,2)
    return size
def del_empty_file(path):
    folders=os.listdir(path)
    for folder in folders:
        folder2=os.listdir(path+'/'+folder)
        if folder2==[]:
            os.rmdir(path+'/'+folder)
def md5sum(fname):
    def read_chunks(fh):
        fh.seek(0)
        chunk=fh.read(8096)
        while chunk:
            yield chunk
            chunk=fh.read(8096)
        else:
            fh.seek(0)
    m=hashlib.md5()
    if isinstance(fname,str) and os.path.exists(fname):
        with open(fname,"rb") as fh:
            for chunk in read_chunks(fh):
                m.update(chunk)
    elif fname.__class__.__name__ in ["StringIO","StringO"] or isinstance(fname,file):
        for chunk in read_chunks(fname):
            m.update(chunk)
    else:
        return ""
    return m.hexdigest()

def rely_(path):
    del_empty_file(path)
    for i in os.listdir(path):
        new_path=path+'/'+i
        #print(new_path)
        for file in os.listdir(new_path):
            new_file=new_path+'/'+file.split(sep='_')[0].split(sep='-')[0]
            if not os.path.exists(new_file):
                os.mkdir(new_file)
                shutil.move(new_path+'/'+file,new_file+'/'+file)
            else:
                if os.path.isfile(new_file):
                    continue
                else:
                    shutil.move(new_path+'/'+file,new_file+'/'+file)
def SELECT(path,db):
    con = sqlite3.connect(db)
    consor = con.cursor()
    md5=md5sum(path)
    values = consor.execute('SELECT * FROM deb WHERE md5=' + '\'' + str(md5) + '\'')
    con.commit()
    db_name=path.split(sep='/')[-1]
    for i in values:
        #and i[4].split(sep='/')[-1]==path.spilt(sep='/')[-1]
        #print(path.spilt(sep='/')[-1])
        #print(i[4].split(sep='/')[-1])
        if i:
            if i[4].split(sep='/')[-1]==path.split(sep='/')[-1]:
                return 0
            else:
                db_name = i[4].split(sep='/')[-1]
    if path.split(sep='/')[-1]!=db_name:
        print(path.split(sep='/')[-1])
        print(db_name)
    return 1

